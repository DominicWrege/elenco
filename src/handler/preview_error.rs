use crate::{hide_internal, model::Permission, template::FeedPreviewSite};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use askama::Template;
use std::error::Error as _;
use std::fmt::Debug;
use thiserror::Error;
use tokio_postgres::error::{DbError, SqlState};
// show session and and parsing error

#[derive(Error, Debug)]
pub enum PreviewError {
    #[error("Invalid RSS Feed {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("Could not fetch URL {0}")]
    Fetch(#[from] reqwest::Error),
    #[error("{0:#?}")]
    Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Podcast {0} already exists.")]
    Duplicate(Field),
}

impl ResponseError for PreviewError {
    fn status_code(&self) -> StatusCode {
        match self {
            PreviewError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self.to_string());
        let message = hide_internal!(PreviewError, self);
        HttpResponse::build(self.status_code())
            .content_type("text/html")
            .body(
                FeedPreviewSite {
                    permission: Some(Permission::User),
                    error_msg: Some(message),
                    context: None,
                }
                .render()
                .unwrap(),
            )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Field {
    #[error("already")]
    Title,
    #[error("URL")]
    Url,
    #[error("img")]
    Img,
}
impl From<tokio_postgres::Error> for PreviewError {
    fn from(error: tokio_postgres::Error) -> Self {
        let source = error.source().and_then(|src| src.downcast_ref::<DbError>());

        if let Some(db_error) = source {
            return match error.code() {
                Some(code) if code == &SqlState::UNIQUE_VIOLATION => match db_error.constraint() {
                    Some(field) if field == "title" => PreviewError::Duplicate(Field::Title),
                    Some(field) if field == "url" => PreviewError::Duplicate(Field::Url),
                    Some(field) if field == "img_path" => PreviewError::Duplicate(Field::Img),
                    _ => PreviewError::Internal(error.into()),
                },
                _ => PreviewError::Internal(error.into()),
            };
        }
        PreviewError::Internal(error.into())
    }
}

impl From<deadpool_postgres::PoolError> for PreviewError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        PreviewError::Internal(e.into())
    }
}

impl From<anyhow::Error> for PreviewError {
    fn from(e: anyhow::Error) -> Self {
        PreviewError::Internal(e.into())
    }
}
