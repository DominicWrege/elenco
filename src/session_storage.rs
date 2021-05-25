use actix_session::Session;

use crate::{
    db,
    model::{Account, Permission},
};

pub const SESSION_KEY_ACCOUNT: &str = "account";
pub const FEED_URL: &str = "feed_url";

#[derive(Debug)]
pub struct SessionContext {
    pub username: String,
    pub permission: Permission,
}

impl SessionContext {
    pub fn from(session: &Session) -> Option<Self> {
        let account = Account::from_session(&session)?;

        Some(Self {
            username: account.username().to_string(),
            permission: account.permission(),
        })
    }
}

pub fn forget(session: &Session) {
    let a = Account::from_session(session);
    dbg!(a);
    session.purge();
    session.remove(SESSION_KEY_ACCOUNT);
}
pub fn cache_feed_url(session: &Session, url: url::Url) -> Result<(), actix_web::Error> {
    session.insert(FEED_URL, url)
}

pub fn feed_url(session: &Session) -> Option<url::Url> {
    session.get::<url::Url>(FEED_URL).ok().flatten()
}
