use actix_session::Session;

pub const SESSION_KEY_ACCOUNT: &str = "account";
pub const FEED_URL: &str = "feed_url";

pub fn forget(session: &Session) {
    // let a = Account::from_session(session);
    // dbg!(a);
    session.purge();
    session.remove(SESSION_KEY_ACCOUNT);
}
pub fn cache_feed_url(session: &Session, url: url::Url) -> Result<(), actix_web::Error> {
    session.insert(FEED_URL, url)
}

pub fn feed_url(session: &Session) -> Option<url::Url> {
    session.get::<url::Url>(FEED_URL).ok().flatten()
}
