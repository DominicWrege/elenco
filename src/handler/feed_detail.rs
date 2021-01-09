use crate::{db::rows_into_vec, inc_sql, model::json::Feed, session_storage, template, State};

use actix_web::{web, HttpResponse};
use askama::Template;
use chrono::{DateTime, Utc};
use template::FeedDetailSite;

use super::general_error::GeneralError;
use crate::time_date::DurationFormator;
use tokio_pg_mapper_derive::PostgresMapper;
#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "episode")]
pub struct EpisodeSmall {
    pub title: String,
    pub duration: Option<i64>,
    pub url: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub explicit: bool,
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
    let client = state.db_pool.get().await?;
    let feed_stmnt = client.prepare(inc_sql!("get/feed/preview_by_id")).await?;
    let feed_row = client.query_one(&feed_stmnt, &[&feed_id]).await?;
    let epsiodes_stmnt = client
        .prepare(inc_sql!("get/episodes_small_for_feed_id"))
        .await?;
    let episodes = rows_into_vec(client.query(&epsiodes_stmnt, &[&feed_id]).await?);
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
