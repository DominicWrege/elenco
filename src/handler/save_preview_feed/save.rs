use actix_broker::{Broker, SystemBroker};
use actix_session::Session;
use actix_web::{web, HttpResponse};

use crate::{
    handler::manage::ModeratorFeed,
    model::{preview::feed::FeedPreview, user::Account},
    socket::Message,
    State,
};

use super::{error::PreviewSaveError, fetch, FeedForm};
use crate::db;

pub async fn save(
    form: web::Json<FeedForm>,
    state: web::Data<State>,
    ses: Session,
) -> Result<HttpResponse, PreviewSaveError> {
    let user_id = Account::from_session(&ses).unwrap().id();
    let mut client = &mut state.db_pool.get().await?;
    let feed_url = form.feed_url.clone();
    let resp_bytes = fetch(&feed_url).await?;
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let raw_feed = FeedPreview::parse(&channel, feed_url);
    let img_cache = state.img_cache.clone();
    let cached_img = if let Some(img_url) = &raw_feed.img {
        img_cache.download(img_url).await.ok()
    } else {
        None
    };
    if db::feed_exits(&client, raw_feed.title, raw_feed.url()).await? {
        return Err(PreviewSaveError::Duplicate(super::error::Field::Url));
    }
    let feed_id = crate::db::feed::save(&mut client, &raw_feed, user_id, cached_img).await?;

    let feed_message = Message::new(ModeratorFeed {
        id: feed_id,
        url: raw_feed.url().to_string(),
        title: raw_feed.title.to_string(),
        author_name: raw_feed.author_name.unwrap_or("default name").to_string(),
        link_web: raw_feed.link_web.map(|u| u.to_string()),
        submitted: chrono::offset::Utc::now(),
        username: Account::from_session(&ses).unwrap().username().to_owned(),
        status: crate::model::Status::Queued,
    });

    Broker::<SystemBroker>::issue_async(feed_message);
    Ok(HttpResponse::Ok().json(SavedJson { feed_id }))
}

#[derive(Debug, serde::Serialize)]
pub struct SavedJson {
    feed_id: i32,
}
