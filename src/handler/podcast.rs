use std::convert::TryFrom;

use crate::{
    db::new_podcast::insert_feed,
    handler::error::HttpError,
    session::{self, SessionStorage},
    template::FeedPreviewSite,
};
use crate::{model::RawFeed, State};
use actix_session::Session;
use actix_web::{http, web, HttpResponse};
use askama_actix::TemplateIntoResponse;
use reqwest::Url;
#[derive(serde::Deserialize)]
pub struct FeedForm {
    #[serde(rename = "feed-url")]
    pub feed: Url,
}

pub async fn save_feed(state: web::Data<State>, ses: Session) -> Result<HttpResponse, HttpError> {
    let user_id = SessionStorage::user_id(&ses);
    let resp_bytes = reqwest::get(session::feed_url(&ses)).await?.text().await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = RawFeed::try_from(&channel)?;
    insert_feed(&mut state.db_pool.get().await.unwrap(), &raw_feed, user_id).await?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/web/profile")
        .finish())
}

pub async fn feed_preview(
    form: web::Form<FeedForm>,
    session: Session,
) -> Result<HttpResponse, HttpError> {
    let url = form.feed.clone();
    let resp_bytes = reqwest::get(url.clone()).await?.bytes().await.unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let preview_site = FeedPreviewSite::preview(&channel, &url);
    session::cache_feed_url(&session, url.clone());
    preview_site
        .into_response()
        .map_err(|e| HttpError::Template(e))
}

pub async fn feed_form() -> Result<HttpResponse, actix_web::Error> {
    FeedPreviewSite::new(None, None).into_response()
}
