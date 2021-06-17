use std::str::FromStr;

use crate::handler::user_feed::UserFeed;
use crate::time_date::{serialize_datetime, serialize_option_datetime};
use crate::Client;
use crate::{handler::api::error::ApiError, util::LanguageCodeLookup};
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

use super::item::MyEnclosure;

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

#[derive(Debug, Serialize)]
pub struct SubmittedFeeds{
    pub blocked: Vec<UserFeed>,
    pub online: Vec<UserFeed>,
    pub offline: Vec<UserFeed>,
    pub queued: Vec<UserFeed>
}



#[derive(Debug, Serialize, PostgresMapper)]
#[pg_mapper(table = "completion")]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    title: String,
    author_name: String,
}

impl Feed {
    pub fn website(&self) -> Option<&Url> {
        self.link_web.as_ref()
    }
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
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub title: String,
    pub description: String,
    pub explicit: bool,
    pub duration: i64,
    pub web_link: Option<String>,
    pub show_notes: Option<String>,
    pub enclosure: MyEnclosure,
    #[serde(serialize_with = "serialize_option_datetime")]
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<String>>,
    pub guid: String,
}

impl From<tokio_postgres::Row> for Episode {
    fn from(row: tokio_postgres::Row) -> Self {
        let media_url: String = row.get("media_url");
        let mime_type: String = row.get("mime_type");

        Self {
            title: row.get("title"),
            description: row.get("description"),
            explicit: row.get("explicit"),
            duration: row.get::<_, Option<i64>>("duration").unwrap_or_default(),
            web_link: row.get("web_link"),
            show_notes: row.get("show_notes"),
            enclosure: MyEnclosure {
                media_url: Url::parse(&media_url).unwrap(),
                length: row.get("media_length"),
                mime_type: mime::Mime::from_str(&mime_type).unwrap(),
            },
            published: row.get("published"),
            keywords: row.get("keywords"),
            guid: row.get("guid"),
        }
    }
}

#[derive(Debug, PostgresMapper, Serialize)]
#[serde(rename_all = "camelCase")]
#[pg_mapper(table = "author")]
pub struct Author {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pub description: String,
    pub children: Vec<SubCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PostgresMapper)]
#[serde(rename_all = "camelCase")]
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
            id,
            description,
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
