pub mod channel;
pub mod item;
pub mod json;
use std::fmt;

use actix_session::Session;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use crate::session_storage::SESSION_KEY_ACCOUNT;

#[derive(Debug, ToSql, FromSql, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[postgres(name = "permission")]
pub enum Permission {
    #[postgres(name = "admin")]
    Admin,
    #[postgres(name = "user")]
    User,
}

impl Default for Permission {
    fn default() -> Self {
        Self::User
    }
}
#[derive(Debug, Deserialize, ToSql, FromSql, PartialEq)]
#[postgres(name = "feed_status")]
pub enum Status {
    #[postgres(name = "online")]
    Online,
    #[postgres(name = "offline")]
    Offline,
    #[postgres(name = "blocked")]
    Blocked,
    #[postgres(name = "queued")]
    Queued,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

impl Account {
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn username(&self) -> &str {
        self.username.as_str()
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
