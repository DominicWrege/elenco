use askama::Template;
use thiserror::Error;

use actix_web::{dev::HttpResponseBuilder, HttpResponse, ResponseError};

use crate::template::FeedPreviewSite;
use actix_web::http::StatusCode;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("An Invalid RSS Feed was provided!. {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("{0}")]
    Connection(#[from] reqwest::Error),
    #[error("DB error: {0}")]
    DB(#[from] anyhow::Error),
    #[error("Template error: {0}")]
    Template(#[from] actix_web::Error),
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
