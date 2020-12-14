use std::convert::TryFrom;

use crate::{db::new_podcast::insert_feed, handler::error::HttpError, template::FeedPreviewSite};
use crate::{model::RawFeed, State};
use actix_web::{http, web, HttpResponse};
use askama::Template;
use askama_actix::TemplateIntoResponse;
use reqwest::Url;
#[derive(serde::Deserialize)]
pub struct FeedForm {
    #[serde(rename = "feed-url")]
    pub feed: Url,
}

pub async fn save_feed(
    form: web::Form<FeedForm>,
    state: web::Data<State>,
    ses: actix_session::Session,
) -> Result<HttpResponse, HttpError> {
    use crate::handler::auth::get_session;
    let user_id = get_session(&ses).unwrap().id;
    let resp_bytes = reqwest::get(form.feed.clone()).await?.text().await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = RawFeed::try_from(&channel)?;
    insert_feed(&mut state.db_pool.get().await.unwrap(), &raw_feed, user_id).await?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/web/profile")
        .finish())
}

pub async fn feed_preview(form: web::Form<FeedForm>) -> Result<HttpResponse, HttpError> {
    let resp_bytes = reqwest::get(form.feed.clone())
        .await?
        .bytes()
        .await
        .unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let preview_site = FeedPreviewSite::preview(&channel, &form.feed);
    preview_site
        .into_response()
        .map_err(|e| HttpError::Template(e))
}

pub async fn feed_form() -> Result<HttpResponse, actix_web::Error> {
    FeedPreviewSite::new(None, None).into_response()
}
