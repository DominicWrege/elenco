use std::convert::{TryFrom, TryInto};

use crate::{inc_sql, model::json::Feed, session_storage, template, State};

use actix_web::{web, HttpResponse};
use askama::Template;
use chrono::{DateTime, Utc};
use reqwest::Url;
use template::FeedDetailSite;

use super::general_error::GeneralError;
use crate::time_date::DurationFormator;
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

pub async fn site(
    state: web::Data<State>,
    session: actix_session::Session,
    id: actix_web::web::Path<i32>,
) -> Result<HttpResponse, GeneralError> {
    let feed_id = id.into_inner();
    dbg!(feed_id);
    let client = state.db_pool.get().await?;
    let feed_stmnt = client.prepare(inc_sql!("get/feed/preview_by_id")).await?;
    let feed_row = client.query_one(&feed_stmnt, &[&feed_id]).await?;
    let epsiodes_stmnt = client
        .prepare(inc_sql!("get/episodes_small_for_feed_id"))
        .await?;
    let episodes = client
        .query(&epsiodes_stmnt, &[&feed_id])
        .await?
        .into_iter()
        .filter_map(|row| row.try_into().ok())
        .collect::<Vec<_>>();
    let feed = Feed::from(&client, feed_row)
        .await
        .map_err(|e| anyhow::format_err!(e))?;
    let html = FeedDetailSite {
        permission: session_storage::permission(&session),
        feed,
        episodes,
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html.render().unwrap()))
}
