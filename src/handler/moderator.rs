use super::{
    auth::{new_account, RegisterError, RegisterForm},
    error::GeneralError,
};
use crate::{
    db::rows_into_vec,
    inc_sql,
    model::Permission,
    template::{self, RegisterModerator},
    util::redirect,
    State,
};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use postgres_types::{FromSql, ToSql};
use serde::Deserialize;

pub async fn manage(state: Data<State>) -> Result<template::ModeratorSite, GeneralError> {
    let client = state.db_pool.get().await?;
    let queued_feeds = client.query(inc_sql!("get/feed/queued"), &[]).await?;
    let reviewed_feed = client
        .query(inc_sql!("get/feed/last_reviewed"), &[])
        .await?;
    Ok(template::ModeratorSite {
        permission: Some(Permission::Admin),
        queued_feeds: rows_into_vec(queued_feeds),
        review_feeds: rows_into_vec(reviewed_feed),
    })
}
#[derive(Debug, Deserialize, ToSql, FromSql)]
#[postgres(name = "feed_status")]
enum Status {
    #[postgres(name = "online")]
    Online,
    #[postgres(name = "offline")]
    Offline,
    #[postgres(name = "blocked")]
    Blocked,
    #[postgres(name = "queued")]
    Queued,
}
#[derive(Debug, Deserialize)]
pub struct Payload {
    action: Status,
    feed_id: i32,
}

pub async fn review_feed(
    json: web::Json<Payload>,
    state: Data<State>,
) -> Result<HttpResponse, GeneralError> {
    let Payload { action, feed_id } = json.into_inner();
    let mut client = state.db_pool.get().await?;
    let trx = client.transaction().await?;
    let stmnt = trx.prepare(inc_sql!("update/review_feed")).await?;
    trx.execute(&stmnt, &[&action, &feed_id]).await?;
    trx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn register(
    form: web::Form<RegisterForm>,
    state: Data<State>,
) -> Result<HttpResponse, RegisterError> {
    let mut client = state.db_pool.get().await?;
    new_account(&mut client, &form, Permission::Admin).await?;
    Ok(redirect("/auth/admin/manage"))
}

pub async fn register_site() -> RegisterModerator {
    RegisterModerator {
        permission: Some(Permission::Admin),
    }
}
