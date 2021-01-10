use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize, Serializer};
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Client;

use crate::{handler::api::error::ApiError, util::LanguageCodeLookup};

#[derive(Debug, Serialize)]
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
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
}

impl Feed {
    pub fn website(&self) -> Option<&Url> {
        self.link_web.as_ref()
    }
}

impl LanguageCodeLookup for Feed {
    fn language_code(&self) -> Option<&str> {
        self.language.as_ref().map(|l| l.as_str())
    }
}

#[derive(Debug, Serialize)]
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
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: Vec<Category>,
    pub epsiodes: Vec<Episode>,
}
#[derive(Debug, PostgresMapper, Serialize)]
#[pg_mapper(table = "episode")]
pub struct Episode {
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

#[derive(Clone, Debug, Serialize)]
pub struct Category {
    id: i32,
    pub description: String,
    pub children: Vec<SubCategory>,
}

#[derive(Clone, Debug, Serialize, PostgresMapper)]
#[pg_mapper(table = "subCategory")]
pub struct SubCategory {
    id: i32,
    pub description: String,
}

impl Category {
    pub fn from(row: &tokio_postgres::Row, children: Vec<SubCategory>) -> Self {
        let id: i32 = row.get("id");
        let description: String = row.get("description");
        Category {
            description,
            id,
            children,
        }
    }
}

fn parse_url(row: &tokio_postgres::Row) -> Option<Url> {
    if let Some(link) = row.get("link_web") {
        Url::parse(link).ok()
    } else {
        None
    }
}

impl FeedEpisode {
    pub async fn from(
        row: &tokio_postgres::Row,
        categories: Vec<Category>,
        epsiodes: Vec<Episode>,
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
            url: Url::parse(feed_row.get("url"))?,
            title: feed_row.get("title"),
            author: feed_row.get("author"),
            img: feed_row.get("img"),
            link_web: parse_url(&feed_row),
            description: feed_row.get("description"),
            subtitle: feed_row.get("subtitle"),
            language: feed_row.get("language"),
            last_modified: feed_row.get("last_modified"),
            categories: categories_for_feed(&client, feed_id).await?,
        })
    }
}
