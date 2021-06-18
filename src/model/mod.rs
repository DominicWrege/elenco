pub mod category;
pub mod json;
pub mod preview;
pub mod user;
use crate::time_date::serialize_datetime;
use chrono::Utc;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio_pg_mapper_derive::PostgresMapper;

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

#[derive(Debug, Serialize, PostgresMapper)]
#[pg_mapper(table = "completion")]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    title: String,
    author_name: String,
}

#[derive(Debug, PostgresMapper, Serialize)]
#[serde(rename_all = "camelCase")]
#[pg_mapper(table = "author")]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Comment {
    id: i32,
    content: String,
    #[serde(serialize_with = "serialize_datetime")]
    created: chrono::DateTime<Utc>,
}
