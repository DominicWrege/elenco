use crate::{
    inc_sql,
    model::{Account, Status},
};
use crate::{template::ProfileSite, State};
use actix_session::Session;
use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use anyhow::anyhow;

use super::general_error::GeneralError;
use crate::db::rows_into_vec;

use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "profilefeed")]
pub struct ProfileFeed {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub img: Option<String>,
    pub author_name: String,
    pub status: Status,
}

pub async fn site(session: Session, state: web::Data<State>) -> Result<ProfileSite, GeneralError> {
    let account = Account::from_session(&session).ok_or_else(|| anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feed/for_profile")).await?;
    let rows = client.query(&stmnt, &[&account.id()]).await?;
    let feeds = rows_into_vec(rows);
    Ok(ProfileSite {
        username: account.username().to_string(),
        permission: Some(account.permission()),
        submitted_feeds: feeds,
    })
}
#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    action: Status,
    feed_id: i32,
}

pub async fn update_feed(
    json: web::Json<Payload>,
    state: Data<State>,
    session: Session,
) -> Result<HttpResponse, GeneralError> {
    let account_id = Account::from_session(&session)
        .ok_or_else(|| anyhow!("session error"))?
        .id();
    let Payload { action, feed_id } = json.into_inner();
    let mut client = state.db_pool.get().await?;
    let trx = client.transaction().await?;
    let stmnt = trx.prepare(inc_sql!("update/profile_update_feed")).await?;
    trx.execute(&stmnt, &[&action, &feed_id, &account_id])
        .await?;
    trx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}
