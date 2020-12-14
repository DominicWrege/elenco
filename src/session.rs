use actix_session::Session;
#[derive(serde::Serialize, Debug, serde::Deserialize)]
pub struct SessionStorage {
    username: String,
    id: i32,
}
const SESSION_KEY_ACCOUNT: &str = "account";
const FEED_URL: &str = "feed_url";

impl SessionStorage {
    pub fn create(session: &Session, username: String, id: i32) -> Result<(), actix_web::Error> {
        session.set(SESSION_KEY_ACCOUNT, Self { username, id })
    }
    pub fn get(session: &Session) -> Option<SessionStorage> {
        session
            .get::<SessionStorage>(SESSION_KEY_ACCOUNT)
            .ok()
            .flatten()
    }
    pub fn forget(session: &Session) {
        session.remove(SESSION_KEY_ACCOUNT);
        session.clear();
    }
    pub fn user_id(session: &Session) -> i32 {
        Self::get(session).unwrap().id
    }
    //FIXME ERROR HANDLING!!!!!!!!!!!!!!!!!!!!!!
    pub fn user(session: &Session) -> (i32, String) {
        let s = Self::get(session).unwrap();
        (s.id, s.username)
    }
}

//FIXME ERROR HANDLING
pub fn cache_feed_url(session: &Session, url: url::Url) {
    session.set(FEED_URL, url).unwrap();
}
//FIXME ERROR HANDLING
pub fn feed_url(session: &Session) -> url::Url {
    session.get::<url::Url>(FEED_URL).unwrap().unwrap()
}
