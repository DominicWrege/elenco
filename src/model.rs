pub mod api;
pub mod episode;
pub mod feed;

use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, ToSql, FromSql, Serialize, Deserialize, Clone, Copy)]
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

#[derive(Debug, PostgresMapper, Serialize, Deserialize, Clone)]
#[pg_mapper(table = "account")]
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
}

fn parse_explicit(it_ext: Option<&rss::extension::itunes::ITunesItemExtension>) -> bool {
    matches!(
        it_ext.and_then(|ext| ext.explicit()),
        Some("Yes") | Some("yes") | Some("true") | Some("True") | Some("explicit")
    )
}
