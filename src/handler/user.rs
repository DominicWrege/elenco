use super::api::error::ApiError;
use crate::db::rows_into_vec;
use crate::model::user::SubmittedFeeds;
use crate::model::feed::TinyFeed;
use crate::State;
use crate::{
    inc_sql,
    model::{user::Account, Status},
};
use actix_session::Session;
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};

fn filter_feeds(feeds: &[TinyFeed], status: Status) -> Vec<TinyFeed> {
    feeds
        .into_iter()
        .filter(|feed| feed.status == status)
        .cloned()
        .map(|feed| feed.to_owned().clone())
        .collect::<Vec<TinyFeed>>()
}

pub async fn submitted_feeds(
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, ApiError> {
    let account = Account::from_session(&session).ok_or_else(|| anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feed/user/submitted")).await?;
    let rows = client.query(&stmnt, &[&account.id()]).await?;
    let feeds: Vec<TinyFeed> = rows_into_vec(rows);
    let feeds_json = SubmittedFeeds {
        blocked: filter_feeds(&feeds, Status::Blocked),
        online: filter_feeds(&feeds, Status::Online),
        offline: filter_feeds(&feeds, Status::Offline),
        queued: filter_feeds(&feeds, Status::Queued),
    };

    Ok(HttpResponse::Ok().json(feeds_json))
}

pub async fn subscriptions(
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, ApiError> {
    let account = Account::from_session(&session).ok_or_else(|| anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let stmnt = client
        .prepare(inc_sql!("get/feed/user/subscription"))
        .await?;

    dbg!(account.id());
    let rows = client.query(&stmnt, &[&account.id()]).await?;
    dbg!(rows.len());
    let feeds: Vec<TinyFeed> = rows_into_vec(rows);
    dbg!(feeds.len());
    Ok(HttpResponse::Ok().json(feeds))
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
) -> Result<HttpResponse, ApiError> {
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
