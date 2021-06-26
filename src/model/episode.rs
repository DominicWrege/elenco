use super::preview::episode::MyEnclosure;
use crate::time_date::serialize_option_datetime;
use crate::time_date::DurationFormator;
use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Serialize;
use std::convert::TryFrom;
use std::str::FromStr;

#[derive(Debug)]
pub struct EpisodeSmall {
    pub title: String,
    pub duration: Option<i64>,
    pub url: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub explicit: bool,
    pub media_url: Url,
}

impl TryFrom<tokio_postgres::Row> for EpisodeSmall {
    type Error = url::ParseError;

    fn try_from(row: tokio_postgres::Row) -> Result<Self, Self::Error> {
        Ok(Self {
            title: row.get("title"),
            duration: row.get("duration"),
            url: row.get("url"),
            published: row.get("published"),
            explicit: row.get("explicit"),
            media_url: Url::parse(row.get("media_url"))?,
        })
    }
}

impl DurationFormator for EpisodeSmall {
    fn duration(&self) -> Option<i64> {
        self.duration
    }
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