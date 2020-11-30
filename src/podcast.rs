use std::convert::{TryFrom, TryInto};

use actix_web::{dev::HttpResponseBuilder, HttpResponse, ResponseError};

use crate::{model::PreviewFeedContent, template::FeedPreviewSite};
use actix_web::http::StatusCode;
use askama::Template;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("An Inavalid RSS Feed was provided!. {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("{0}")]
    Connection(#[from] reqwest::Error),
    #[error("DB error: {0}")]
    DB(#[from] anyhow::Error),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .content_type("text/html")
            .body(
                FeedPreviewSite::new(None, Some(self.to_string()))
                    .render()
                    .unwrap(),
            )
    }
}
// TODO better err handling

pub fn parse_img_url(feed: &rss::Channel) -> Option<Url> {
    feed.image()
        .and_then(|img| Url::parse(img.url()).ok())
        .or_else(|| {
            feed.itunes_ext()
                .and_then(|it| it.image().and_then(|u| Url::parse(u).ok()))
        })
}
fn parse_author(feed: &rss::Channel) -> String {
    feed.itunes_ext()
        .and_then(|x| x.author())
        .unwrap_or_default()
        .into()
}

pub fn episode_list<'a, T>(feed: &'a rss::Channel) -> Vec<T>
where
    T: TryFrom<&'a rss::Item>,
{
    feed.items()
        .iter()
        .flat_map(|item| item.try_into().ok())
        .collect()
}

impl<'a> From<&'a rss::Channel> for FeedPreviewSite<'a> {
    fn from(feed: &'a rss::Channel) -> Self {
        FeedPreviewSite::new(
            Some(PreviewFeedContent {
                url: Url::parse(feed.link()).unwrap(),
                img: parse_img_url(&feed),
                title: feed.title(),
                description: feed.description(),
                author: parse_author(&feed),
                episodes: episode_list(&feed),
            }),
            None,
        )
    }
}

pub mod handler {
    use std::convert::TryFrom;

    use actix_web::{http, web, HttpResponse};

    use super::{FeedPreviewSite, HttpError, Url};
    use crate::{db::insert_feed, model::RawFeed, State};
    use askama::Template;
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
        use crate::auth::get_session;
        let user_id = get_session(&ses).unwrap().id;
        let resp_bytes = reqwest::get(form.feed.clone()).await?.bytes().await?;
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
        let preview_site = FeedPreviewSite::from(&channel);

        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(preview_site.render().unwrap()))
    }

    pub async fn feed_form() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html")
            .body(FeedPreviewSite::new(None, None).render().unwrap())
    }
}
