use actix_session::Session;

use crate::model::{Account, Permission};

const SESSION_KEY_ACCOUNT: &str = "account";
const FEED_URL: &str = "feed_url";

impl Account {
    pub fn save(&self, session: &Session) -> Result<(), actix_web::Error> {
        session.set(SESSION_KEY_ACCOUNT, self)
    }
    pub fn get_account(session: &Session) -> Option<Account> {
        session.get::<Account>(SESSION_KEY_ACCOUNT).ok().flatten()
    }
}
pub fn forget(session: &Session) {
    session.purge();
    session.remove(SESSION_KEY_ACCOUNT);
}
pub fn permission(session: &Session) -> Option<Permission> {
    Account::get_account(&session).map(|a| a.permission)
}
pub fn cache_feed_url(session: &Session, url: url::Url) -> Result<(), actix_web::Error> {
    session.set(FEED_URL, url)
}

pub fn feed_url(session: &Session) -> Option<url::Url> {
    session.get::<url::Url>(FEED_URL).ok().flatten()
}
