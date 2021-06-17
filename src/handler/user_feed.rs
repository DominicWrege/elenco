use super::general_error::GeneralError;
use crate::db::rows_into_vec;
use crate::model::json::SubmittedFeeds;
use crate::State;
use crate::{
    inc_sql,
    model::{Account, Status},
};
use actix_session::Session;
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use ammonia::AttributeFilter;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "profilefeed")]
#[serde(rename_all = "camelCase")]
pub struct UserFeed {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub img: Option<String>,
    pub author_name: String,
    pub status: Status,
}

fn filter_feeds(feeds: &[UserFeed], status: Status) -> Vec<UserFeed> {
    feeds
        .into_iter()
        .filter(|feed| feed.status == status)
        .cloned()
        .map(|feed| feed.to_owned().clone())
        .collect::<Vec<UserFeed>>()
}

pub async fn site(session: Session, state: web::Data<State>) -> Result<HttpResponse, GeneralError> {
    let account = Account::from_session(&session).ok_or_else(|| anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feed/for_profile")).await?;
    let rows = client.query(&stmnt, &[&account.id()]).await?;
    let feeds: Vec<UserFeed> = rows_into_vec(rows);
    let feeds_json = SubmittedFeeds {
        blocked: filter_feeds(&feeds, Status::Blocked),
        online: filter_feeds(&feeds, Status::Online),
        offline: filter_feeds(&feeds, Status::Offline),
        queued: filter_feeds(&feeds, Status::Queued),
    };

    Ok(HttpResponse::Ok().json(feeds_json))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePayload {
    action: Status,
    feed_id: i32,
}

pub async fn update_feed(
    json: web::Json<UpdatePayload>,
    state: Data<State>,
    session: Session,
) -> Result<HttpResponse, GeneralError> {
    let account_id = Account::from_session(&session)
        .ok_or_else(|| anyhow!("session error"))?
        .id();
    let UpdatePayload { action, feed_id } = json.into_inner();
    let mut client = state.db_pool.get().await?;
    let trx = client.transaction().await?;
    let stmnt = trx.prepare(inc_sql!("update/profile_update_feed")).await?;
    trx.execute(&stmnt, &[&action, &feed_id, &account_id])
        .await?;
    trx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
