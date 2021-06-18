use actix_session::Session;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use crate::session_storage::SESSION_KEY_ACCOUNT;

use super::{Permission, Status};

#[derive(Debug, PostgresMapper, Serialize, Deserialize, Clone)]
#[pg_mapper(table = "account")]
#[serde(rename_all = "camelCase")]
pub struct Account {
    username: String,
    email: String,
    #[serde(skip_serializing, skip_deserializing)]
    password_hash: String,
    id: i32,
    pub permission: Permission,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShortAccount {
    pub id: i32,
    pub username: String,
}

impl Account {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn permission(&self) -> Permission {
        self.permission
    }
    pub fn password_hash(&self) -> &str {
        self.password_hash.as_str()
    }
    pub fn save(&self, session: &Session) -> Result<(), actix_web::Error> {
        session.insert(SESSION_KEY_ACCOUNT, self)
    }
    pub fn from_session(session: &Session) -> Option<Account> {
        session.get::<Account>(SESSION_KEY_ACCOUNT).ok().flatten()
    }
}

#[derive(Debug, Serialize)]
pub struct SubmittedFeeds {
    pub blocked: Vec<UserFeed>,
    pub online: Vec<UserFeed>,
    pub offline: Vec<UserFeed>,
    pub queued: Vec<UserFeed>,
}

#[derive(Debug, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "profilefeed")]
#[serde(rename_all = "camelCase")]
pub struct UserFeed {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub description: String,
    pub img: Option<String>,
    pub author_name: String,
    pub status: Status,
}
