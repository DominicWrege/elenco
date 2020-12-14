use std::convert::{TryFrom, TryInto};

use actix_web::{dev::HttpResponseBuilder, HttpResponse, ResponseError};

use crate::template::FeedPreviewSite;
use actix_web::http::StatusCode;
use askama::Template;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("An Invalid RSS Feed was provided!. {0}")]
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
        log::error!("{:#?}", &self);
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
pub fn parse_author(feed: &rss::Channel) -> String {
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
