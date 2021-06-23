use crate::hide_internal;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use std::error::Error as _;
use std::fmt::Debug;
use thiserror::Error;
use tokio_postgres::error::{DbError, SqlState};

// show session and and parsing error

#[derive(Error, Debug)]
pub enum PreviewSaveError {
    #[error("Invalid RSS Feed {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("Could not fetch from URL {0}")]
    Fetch(url::Url),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("{0:#?}")]
    Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("Can't save the the RSS-feed because this unique field {0} already exists.")]
    Duplicate(Field),
}

impl ResponseError for PreviewSaveError {
    fn status_code(&self) -> StatusCode {
        match self {
            PreviewSaveError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self.to_string());
        // error::JsonError::into_response(hide_internal!(PreviewError, self), self.status_code())
        crate::json_error!(PreviewSaveError, self)
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

impl From<tokio_postgres::Error> for PreviewSaveError {
    fn from(error: tokio_postgres::Error) -> Self {
        let source = error.source().and_then(|src| src.downcast_ref::<DbError>());

        if let Some(db_error) = source {
            return match error.code() {
                Some(code) if code == &SqlState::UNIQUE_VIOLATION => match db_error.constraint() {
                    Some(field) if field == "title" => PreviewSaveError::Duplicate(Field::Title),
                    Some(field) if field == "url" => PreviewSaveError::Duplicate(Field::Url),
                    Some(field) if field == "img_path" => PreviewSaveError::Duplicate(Field::Img),
                    _ => PreviewSaveError::Internal(error.into()),
                },
                _ => PreviewSaveError::Internal(error.into()),
            };
        }
        PreviewSaveError::Internal(error.into())
    }
}

impl From<deadpool_postgres::PoolError> for PreviewSaveError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        PreviewSaveError::Internal(e.into())
    }
}

impl From<anyhow::Error> for PreviewSaveError {
    fn from(e: anyhow::Error) -> Self {
        PreviewSaveError::Internal(e.into())
    }
}
