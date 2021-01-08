use crate::{
    db::{feed_exits, save_feed::save, preview_error::PreviewError},
    model::{channel::RawFeed, Account},
    session_storage::{cache_feed_url, feed_url},
    template::{Context, FeedPreviewSite},
    util::redirect,
    State,
};

use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;
use reqwest::Url;

#[derive(serde::Deserialize)]
pub struct FeedForm {
    #[serde(rename = "feed-url")]
    pub feed: Url,
}

pub async fn save_feed(
    state: web::Data<State>,
    ses: Session,
) -> Result<HttpResponse, PreviewError> {
    let user_id = Account::from_session(&ses).unwrap().id();
    let feed_url =
        feed_url(&ses).ok_or_else(|| anyhow::anyhow!("session error: cache_feed_url not found"))?;
    let resp_bytes = reqwest::get(feed_url.clone()).await?.text().await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = RawFeed::parse(&channel, feed_url)?;
    let img_cache = state.img_cache.clone();
    let cached_img = if let Some(img_url) = &raw_feed.img {
        img_cache.download(img_url).await.ok()
    } else {
        None
    };

    save(
        &mut state.db_pool.get().await?,
        &raw_feed,
        user_id,
        cached_img,
    )
    .await?;

    Ok(redirect("/auth/profile"))
}

pub async fn create_preview(
    form: web::Form<FeedForm>,
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, PreviewError> {
    let url = form.feed.clone();
    let resp_bytes = reqwest::get(url.clone()).await?.bytes().await.unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    cache_feed_url(&session, url.clone()).map_err(|_| anyhow::anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let raw_feed = RawFeed::parse(&channel, url.clone())?;

    let context = Context {
        feed_exists: feed_exits(&client, raw_feed.title, raw_feed.url()).await?,
        feed: raw_feed,
    };

    let template = FeedPreviewSite {
        permission: Account::from_session(&session).map(|acount| acount.permission()),
        error_msg: None,
        context: Some(context),
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok().content_type("text/html").body(template))
}

pub async fn form_template<'a>(session: Session) -> Result<FeedPreviewSite<'a>, actix_web::Error> {
    Ok(FeedPreviewSite {
        permission: Account::from_session(&session).map(|acount| acount.permission()),
        error_msg: None,
        context: None,
    })
}
