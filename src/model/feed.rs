use crate::time_date::serialize_datetime;
use crate::Client;
use crate::{handler::error::ApiError, util::LanguageCodeLookup};
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Serialize;

use super::category::Category;
use crate::model::episode::Episode;

use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: i32,
    pub url: Url,
    pub title: String,
    pub img: Option<String>,
    pub author: String,
    pub link_web: Option<Url>,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: Option<String>,
    pub img_cache: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
}

impl LanguageCodeLookup for Feed {
    fn language_code(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedEpisode {
    pub id: i32,
    pub url: Url,
    pub title: String,
    pub author: String,
    pub img: Option<String>,
    pub link_web: Option<Url>,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: Option<String>,
    pub img_cache: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
    pub episodes: Vec<Episode>,
}

impl FeedEpisode {
    pub async fn from(
        row: &tokio_postgres::Row,
        categories: Vec<Category>,
        episodes: Vec<Episode>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            id: row.get("id"),
            url: Url::parse(row.get("url"))?,
            title: row.get("title"),
            author: row.get("author"),
            img: row.get("img"),
            link_web: parse_url(&row),
            description: row.get("description"),
            subtitle: row.get("subtitle"),
            language: row.get("language"),
            last_modified: row.get("last_modified"),
            img_cache: row.get("img_cache"),
            categories,
            episodes,
        })
    }
}

impl Feed {
    pub async fn from(client: &Client, feed_row: tokio_postgres::Row) -> Result<Self, ApiError> {
        use crate::db::category::get_categories_for_feed;
        let feed_id = feed_row.get("id");
        Ok(Self {
            id: feed_id,
            url: Url::parse(feed_row.get("url"))?,
            title: feed_row.get("title"),
            author: feed_row.get("author"),
            img: feed_row.get("img"),
            link_web: parse_url(&feed_row),
            description: feed_row.get("description"),
            subtitle: feed_row.get("subtitle"),
            language: feed_row.get("language"),
            img_cache: feed_row.get("img_cache"),
            last_modified: feed_row.get("last_modified"),
            categories: get_categories_for_feed(&client, feed_id).await?,
        })
    }
}

fn parse_url(row: &tokio_postgres::Row) -> Option<Url> {
    if let Some(link) = row.get("link_web") {
        Url::parse(link).ok()
    } else {
        None
    }
}

#[derive(Debug, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "profilefeed")]
#[serde(rename_all = "camelCase")]
pub struct TinyFeed {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub img: Option<String>,
    pub author_name: String,
    pub status: super::Status,
    #[serde(serialize_with = "serialize_datetime")]
    pub submitted: DateTime<Utc>,
}
