use super::{
    auth::{new_account, RegisterError, RegisterForm},
    general_error::GeneralError,
};
use crate::{
    db::rows_into_vec,
    inc_sql,
    model::{Permission, Status},
    template::{self, RegisterModerator},
    util::redirect,
    State,
};
use actix_web::{
    web::{self, Data},
    HttpResponse,
};

use chrono::{DateTime, Utc};

use serde::Deserialize;
use tokio_pg_mapper_derive::PostgresMapper;

pub async fn manage(state: Data<State>) -> Result<template::ModeratorSite, GeneralError> {
    let client = state.db_pool.get().await?;
    let queued_feed_rows = client.query(inc_sql!("get/feed/queued"), &[]).await?;
    let reviewed_feed_rows = client
        .query(inc_sql!("get/feed/last_reviewed"), &[])
        .await?;
    Ok(template::ModeratorSite {
        permission: Some(Permission::Admin),
        queued_feeds: rows_into_vec(queued_feed_rows),
        review_feeds: rows_into_vec(reviewed_feed_rows),
    })
}

#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "feed")]
pub struct ModeratorFeed {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub img_cache: Option<String>,
    pub author_name: String,
    pub link_web: Option<String>,
    pub status: Status,
    pub submitted: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub language: String,
    pub username: String,
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
