use crate::{
    db::new_podcast::insert_feed,
    model::{Account, Permission},
    session::{cache_feed_url, feed_url},
    template::FeedPreviewSite,
    util::redirect,
};
use crate::{model::RawFeed, State};
use actix_session::Session;
use actix_web::{http, web, HttpResponse, ResponseError};
use askama::Template;
use reqwest::Url;
use thiserror::Error;

use actix_web::http::StatusCode;

// TODO some anyhow::Eror -> parse error ??
// FIX ME better error
#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("An Invalid RSS Feed was provided!. {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("{0}")]
    Connection(#[from] reqwest::Error),
    #[error("DB error: {0}")]
    General(#[from] anyhow::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
}

impl ResponseError for PreviewError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{:#?}", &self);
        HttpResponse::build(self.status_code())
            .content_type("text/html")
            .body(
                FeedPreviewSite {
                    metadata: None,
                    permission: Some(Permission::User),
                    error_msg: Some(self.to_string()),
                }
                .render()
                .unwrap(),
            )
    }
}

#[derive(serde::Deserialize)]
pub struct FeedForm {
    #[serde(rename = "feed-url")]
    pub feed: Url,
}

pub async fn save_feed(
    state: web::Data<State>,
    ses: Session,
) -> Result<HttpResponse, PreviewError> {
    let user_id = Account::get_account(&ses).unwrap().id();
    let feed_url = feed_url(&ses).unwrap();
    let resp_bytes = reqwest::get(feed_url.clone()).await?.text().await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = RawFeed::parse(&channel, feed_url)?;
    insert_feed(&mut state.db_pool.get().await?, &raw_feed, user_id).await?;

    Ok(redirect("/auth/profile"))
}

pub async fn feed_preview(
    form: web::Form<FeedForm>,
    session: Session,
) -> Result<HttpResponse, PreviewError> {
    let url = form.feed.clone();
    let resp_bytes = reqwest::get(url.clone()).await?.bytes().await.unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    cache_feed_url(&session, url.clone())
        .map_err(|_| PreviewError::General(anyhow::anyhow!("cache feed, session error ..")))?;

    let template = FeedPreviewSite {
        metadata: Some(RawFeed::parse(&channel, url.clone())?),
        permission: Account::get_account(&session).map(|acount| acount.permission()),
        error_msg: None,
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().body(template))
}

pub async fn feed_form<'a>(session: Session) -> Result<FeedPreviewSite<'a>, actix_web::Error> {
    Ok(FeedPreviewSite {
        metadata: None,
        permission: Account::get_account(&session).map(|acount| acount.permission()),
        error_msg: None,
    })
}
