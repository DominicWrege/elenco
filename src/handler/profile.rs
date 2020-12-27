use crate::{inc_sql, model::Account};
use crate::{template::ProfileSite, State};
use actix_session::Session;
use actix_web::web;

use super::error::GeneralError;
use crate::db::rows_into_vec;
pub async fn site(session: Session, state: web::Data<State>) -> Result<ProfileSite, GeneralError> {
    let account = Account::get_account(&session).unwrap();
    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feeds_for_account")).await?;
    let rows = client.query(&stmnt, &[&account.id()]).await?;
    let feeds = rows_into_vec(rows);
    Ok(ProfileSite {
        username: account.username().to_string(),
        permission: Some(account.permission()),
        submitted_feeds: feeds,
    })
}
