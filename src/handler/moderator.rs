use super::{
    auth::{new_account, RegisterError, RegisterForm},
    error::GeneralError,
};
use crate::{
    db::admin::{queued_feeds, reviewed_feed},
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
    let mut client = state.db_pool.get().await?;
    Ok(template::ModeratorSite {
        permission: Some(Permission::Admin),
        queued_feeds: queued_feeds(&mut client).await?,
        review_feeds: reviewed_feed(&mut client).await?,
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
