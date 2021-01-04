use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize, Serializer};
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Client;

use crate::handler::api::error::ApiError;

use super::channel::LanguageCodeLookup;
#[derive(Debug, Serialize)]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub img: Option<String>,
    pub author: String,
    pub link_web: String,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
}

impl Feed {
    pub fn website(&self) -> Option<Url> {
        Url::parse(&self.link_web).ok()
    }
}

impl LanguageCodeLookup for Feed {
    fn language_code(&self) -> Option<&str> {
        self.language.as_ref().map(|l| l.as_str())
    }
}

#[derive(Debug, Serialize)]
pub struct FeedEpsiode {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub author: String,
    pub img: Option<String>,
    pub link_web: String,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
    pub epsiodes: Vec<Epsiode>,
}
#[derive(Debug, PostgresMapper, Serialize)]
#[pg_mapper(table = "epsiode")]
pub struct Epsiode {
    pub title: String,
    pub description: String,
    pub explicit: bool,
    pub duration: i64,
    pub web_link: Option<String>,
    pub show_notes: Option<String>,
    pub media_url: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub published: DateTime<Utc>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, PostgresMapper, Serialize)]
#[pg_mapper(table = "author")]
pub struct Author {
    pub id: i32,
    pub name: String,
}

pub fn serialize_datetime<S>(date: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    return s.serialize_str(&date.to_rfc3339().to_string());
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    id: i32,
    pub description: String,
    pub childreen: Vec<SubCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubCategory {
    id: i32,
    pub description: String,
}

impl FeedEpsiode {
    pub async fn from(
        row: &tokio_postgres::Row,
        categories: Vec<Category>,
        epsiodes: Vec<Epsiode>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            author: row.get("author"),
            img: row.get("img"),
            link_web: row.get("link_web"),
            description: row.get("description"),
            subtitle: row.get("subtitle"),
            language: row.get("language"),
            last_modified: row.get("last_modified"),
            categories: categories,
            epsiodes: epsiodes,
        })
    }
}

impl Feed {
    pub async fn from(client: &Client, feed_row: tokio_postgres::Row) -> Result<Self, ApiError> {
        use crate::db::categories_for_feed;
        let feed_id = feed_row.get("id");
        Ok(Self {
            id: feed_id,
            url: feed_row.get("url"),
            title: feed_row.get("title"),
            author: feed_row.get("author"),
            img: feed_row.get("img"),
            link_web: feed_row.get("link_web"),
            description: feed_row.get("description"),
            subtitle: feed_row.get("subtitle"),
            language: feed_row.get("language"),
            last_modified: feed_row.get("last_modified"),
            categories: categories_for_feed(&client, feed_id).await?,
        })
    }
    pub fn compare_subtile_description(&self) -> bool {
        self.subtitle
            .as_ref()
            .map(|subtitle| subtitle == &self.description)
            .is_some()
    }
}

impl From<tokio_postgres::Row> for Category {
    fn from(row: tokio_postgres::Row) -> Self {
        let id: i32 = row.get("id");
        let description: String = row.get("description");
        Category {
            description,
            id,
            childreen: serde_json::from_value(row.get("subcategories")).unwrap(),
        }
    }
}
