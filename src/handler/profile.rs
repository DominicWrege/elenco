use crate::{
    inc_sql,
    model::{Account, Status},
};
use crate::{template::ProfileSite, State};
use actix_session::Session;
use actix_web::web;

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
    let account = Account::from_session(&session).unwrap();
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
