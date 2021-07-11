use crate::db::category::get_categories_for_feed;
use crate::time_date::serialize_datetime;
use crate::{db, Client};
use crate::{handler::error::ApiError, util::LanguageCodeLookup};
use chrono::{DateTime, Utc};
use db::episode;
use episode::episode_offset;
use reqwest::Url;
use serde::Serialize;

use super::category::Category;
use super::preview::episode::{Episode, EpisodeNext};

use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: i32,
    pub url: Url,
    pub title: String,
    pub img: Option<String>,
    pub author_name: String,
    pub link_web: Option<Url>,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: Option<String>,
    pub img_cache: Option<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub submitted: DateTime<Utc>,
    pub categories: Vec<Category>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<EpisodeNext>,
}

impl LanguageCodeLookup for Feed {
    fn language_code(&self) -> Option<&str> {
        self.language.as_deref()
    }
}

impl Feed {
    pub async fn from(
        client: &Client,
        row: tokio_postgres::Row,
        episodes: Option<Vec<Episode>>,
    ) -> Result<Self, ApiError> {
        let id = row.get("id");

        let epsiodes_next = if let Some(items) = episodes {
            Some(EpisodeNext {
                offset: episode_offset(&client, &items, &id).await?,
                items,
            })
        } else {
            None
        };

        Ok(Self {
            id,
            url: Url::parse(row.get("url"))?,
            title: row.get("title"),
            author_name: row.get("author_name"),
            img: row.get("img"),
            link_web: parse_url(&row),
            description: row.get("description"),
            subtitle: row.get("subtitle"),
            language: row.get("language"),
            submitted: row.get("submitted"),
            img_cache: row.get("img_cache"),
            categories: get_categories_for_feed(&client, id).await?,
            episodes: epsiodes_next,
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
