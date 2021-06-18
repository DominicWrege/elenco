use actix_session::Session;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use crate::session_storage::SESSION_KEY_ACCOUNT;

use super::Permission;

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
