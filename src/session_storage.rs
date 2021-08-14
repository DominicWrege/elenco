use actix_session::Session;

pub const SESSION_KEY_ACCOUNT: &str = "account";
//pub const FEED_URL: &str = "feed_url";

pub fn forget(session: &Session) {

    session.purge();
    session.remove(SESSION_KEY_ACCOUNT);
}
