use crate::{
    db::{feed_exits, save_feed::save},
    model::{channel::Feed, Account},
    session_storage::{cache_feed_url, feed_url},
    socket::MessageRowHtml,
    template::{Context, FeedPreviewSite, ModeratorFeedTableRow},
    util::redirect,
    State,
};

use actix_broker::{Broker, SystemBroker};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;
use reqwest::Url;

use super::preview_error::PreviewError;

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
    let resp_bytes = fetch(&feed_url).await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = Feed::parse(&channel, feed_url);
    let img_cache = state.img_cache.clone();
    let cached_img = if let Some(img_url) = &raw_feed.img {
        img_cache.download(img_url).await.ok()
    } else {
        None
    };

    let feed_id = save(
        &mut state.db_pool.get().await?,
        &raw_feed,
        user_id,
        cached_img,
    )
    .await?;

    let feed_tr = ModeratorFeedTableRow {
        id: feed_id,
        url: raw_feed.url().to_string(),
        title: raw_feed.title.to_string(),
        author_name: raw_feed
            .author
            .unwrap_or_else(|| "default name")
            .to_string(),
        link_web: raw_feed.link_web.map(|u| u.to_string()),
        submitted: chrono::offset::Utc::now(),
        last_modified: chrono::offset::Utc::now(),
        username: Account::from_session(&ses).unwrap().username().to_string(),
    }
    .render()
    .unwrap();

    Broker::<SystemBroker>::issue_async(MessageRowHtml::new(feed_tr));

    Ok(redirect("/auth/profile"))
}

pub async fn create_preview(
    form: web::Form<FeedForm>,
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, PreviewError> {
    let resp_bytes = fetch(&form.feed).await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let url = form.feed.clone();
    cache_feed_url(&session, url.clone()).map_err(|_| anyhow::anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let raw_feed = Feed::parse(&channel, url);

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

async fn fetch(url: &Url) -> Result<web::Bytes, PreviewError> {
    let bytes = reqwest::get(url.clone())
        .await
        .map_err(|_err| PreviewError::Fetch(url.clone()))?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes)
}

pub async fn form_template<'a>(session: Session) -> Result<FeedPreviewSite<'a>, actix_web::Error> {
    Ok(FeedPreviewSite {
        permission: Account::from_session(&session).map(|acount| acount.permission()),
        error_msg: None,
        context: None,
    })
}
