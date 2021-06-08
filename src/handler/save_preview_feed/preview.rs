use crate::{db::feed_exits, model::channel::Feed, session_storage::cache_feed_url, State};

// use actix_broker::{Broker, SystemBroker};
use actix_session::Session;
use actix_web::{web, HttpResponse};

use reqwest::Url;

use super::{error::PreviewSaveError, FeedForm};

pub async fn create(
    form: web::Json<FeedForm>,
    session: Session,
    state: web::Data<State>,
) -> Result<HttpResponse, PreviewSaveError> {
    let resp_bytes = super::fetch(&form.feed_url).await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    // let url = form.feed_url.clone();
    // cache_feed_url(&session, url.clone()).map_err(|_| anyhow::anyhow!("session error"))?;
    let client = state.db_pool.get().await?;
    let raw_feed = Feed::parse(&channel, form.feed_url.clone());

    Ok(HttpResponse::Ok().json(PreviewJson {
        exists: feed_exits(&client, raw_feed.title, raw_feed.url()).await?,
        feed: raw_feed,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct PreviewJson<'a> {
    pub exists: bool,
    pub feed: Feed<'a>,
}

// pub async fn form_template<'a>(session: Session) -> Result<HttpResponse, actix_web::Error> {
//     // Ok(FeedPreviewSite {
//     //     session_context: SessionContext::from(&session),
//     //     error_msg: None,
//     //     context: None,
//     // })
//     todo!()
// }
