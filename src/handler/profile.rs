use crate::model::Account;
use crate::{db::api::get_feeds_for_account, template::ProfileSite, State};
use actix_session::Session;
use actix_web::web;

use super::error::GeneralError;

pub async fn site(session: Session, state: web::Data<State>) -> Result<ProfileSite, GeneralError> {
    let account = Account::get_account(&session).unwrap();
    let pool = state.db_pool.get().await?;
    Ok(ProfileSite {
        username: account.username().to_string(),
        permission: Some(account.permission()),
        submitted_feeds: get_feeds_for_account(&pool, account.id()).await?,
    })
}
