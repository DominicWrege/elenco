use crate::model::user::Account;
use crate::time_date::serialize_datetime;
use crate::{
    auth::{
        error::AuthError,
        register::{self, RegisterForm},
    },
    db::rows_into_vec,
    inc_sql,
    model::{Permission, Status},
    socket::LiveFeedSocket,
    util::redirect,
    State,
};
use actix_web::{
    web::{self, Data, Json},
    HttpResponse,
};
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use super::{error::ApiError, ApiJsonResult};

pub async fn all_unassigned(state: Data<State>) -> ApiJsonResult<Vec<ModeratorFeed>> {
    let client = state.db_pool.get().await?;
    let queued_feed_rows = client
        .query(inc_sql!("get/feed/moderator/all_waiting_for_review"), &[])
        .await?;
    // let reviewed_feed_rows = client
    //     .query(inc_sql!("get/feed/last_reviewed"), &[])
    //     .await?;
    // // Ok(template::ModeratorSite {
    // //     session_context: SessionContext::from(&session),
    // //     queued_feeds: rows_into_vec(queued_feed_rows),
    // //     review_feeds: rows_into_vec(reviewed_feed_rows),
    // //     username: "test".into(),
    // // })

    Ok(Json(rows_into_vec(queued_feed_rows)))
}

#[derive(Debug, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "feed")]
#[serde(rename_all = "camelCase")]
pub struct ModeratorFeed {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub author_name: String,
    pub link_web: Option<String>,
    pub status: Status,
    #[serde(serialize_with = "serialize_datetime")]
    pub submitted: DateTime<Utc>,
    pub username: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewPayload {
    action: Status,
    feed_ids: Vec<i32>,
}

pub async fn review_feed(
    json: web::Json<ReviewPayload>,
    state: Data<State>,
) -> Result<HttpResponse, ApiError> {
    let ReviewPayload { action, feed_ids } = json.into_inner();
    let mut client = state.db_pool.get().await?;
    let trx = client.transaction().await?;
    let stmnt_review_feed = trx.prepare(inc_sql!("update/review_feed")).await?;
    let stmnt_update_review = trx.prepare(inc_sql!("update/done_review")).await?;
    for feed_id in feed_ids {
        trx.execute(&stmnt_review_feed, &[&action, &feed_id])
            .await?;
        trx.execute(&stmnt_update_review, &[&feed_id]).await?;
    }
    trx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignPayload {
    feed_ids: Vec<i32>,
}

pub async fn reviewer_inbox(
    session: actix_session::Session,
    state: Data<State>,
) -> ApiJsonResult<Vec<ModeratorFeed>> {
    let user_id = Account::from_session(&session)
        .map(|a| a.id())
        .ok_or_else(|| anyhow::anyhow!("Session Error"))?;

    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feed/moderator/inbox")).await?;
    let rows = client.query(&stmnt, &[&user_id]).await?;

    Ok(Json(rows_into_vec(rows)))
}

pub async fn assign_for_review(
    session: actix_session::Session,
    state: Data<State>,
    json: web::Json<AssignPayload>,
) -> Result<HttpResponse, ApiError> {
    let mut client = state.db_pool.get().await?;
    let trx = client.transaction().await?;

    let user_id = Account::from_session(&session)
        .map(|a| a.id())
        .ok_or_else(|| anyhow::anyhow!("Session Error"))?;
    let stmnt = trx.prepare(inc_sql!("/update/assign_for_review")).await?;
    for feed_id in &json.feed_ids {
        trx.execute(&stmnt, &[&user_id, feed_id]).await?;
    }
    trx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn register_moderator(
    form: web::Form<RegisterForm>,
    state: Data<State>,
) -> Result<HttpResponse, AuthError> {
    let mut client = state.db_pool.get().await?;
    register::new_account(&mut client, &form, Permission::Admin).await?;
    Ok(redirect("/auth/admin/manage"))
}

pub async fn register_socket(
    req: web::HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let resp = actix_web_actors::ws::start(LiveFeedSocket::new(), &req, stream)?;
    Ok(resp)
}
