pub mod category;

pub mod episode;
pub mod feed;
pub mod preview;
pub mod user;

use crate::time_date::serialize_datetime;
use chrono::Utc;
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;

use self::user::ShortAccount;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewComment {
    pub user_id: i32,
    pub feed_id: i32,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    id: i32,
    feed_id: i32,
    content: String,
    #[serde(serialize_with = "serialize_datetime")]
    created: chrono::DateTime<Utc>,
    user: ShortAccount,
}

impl From<Row> for Comment {
    fn from(row: Row) -> Self {
        Self {
            id: row.get("id"),
            feed_id: row.get("feed_id"),
            content: row.get("content"),
            created: row.get("created"),
            user: ShortAccount {
                id: row.get("user_id"),
                username: row.get("username"),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub episodes_duration: i64,
    pub count_episodes: i64,
    pub count_authors: i64,
    pub count_feeds: i64,
}
