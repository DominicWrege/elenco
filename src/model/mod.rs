pub mod channel;
pub mod item;
pub mod json;
pub mod user;
use std::fmt;

use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, ToSql, FromSql, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[postgres(name = "permission")]
pub enum Permission {
    #[postgres(name = "admin")]
    Admin,
    #[postgres(name = "user")]
    User,
}

#[derive(Debug, Deserialize, Serialize, ToSql, FromSql, PartialEq, Clone)]
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
