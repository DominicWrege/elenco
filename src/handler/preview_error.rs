use crate::hide_internal;
use actix_web::{body::Body, ResponseError};
use actix_web::{http::StatusCode, BaseHttpResponse};
use api::error;
use std::error::Error as _;
use std::fmt::Debug;
use thiserror::Error;
use tokio_postgres::error::{DbError, SqlState};

use super::api;
// show session and and parsing error

#[derive(Error, Debug)]
pub enum PreviewError {
    #[error("Invalid RSS Feed {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("Could not fetch from URL {0}")]
    Fetch(url::Url),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("{0:#?}")]
    Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("RSS-Feed {0} already exists.")]
    Exists(String),
    #[error("Can't save the the RSS-feed because this unique field {0} already exists.")]
    Duplicate(Field),
}

impl ResponseError for PreviewError {
    fn status_code(&self) -> StatusCode {
        match self {
            PreviewError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{}", self.to_string());
        // error::JsonError::into_response(hide_internal!(PreviewError, self), self.status_code())
        crate::json_error!(PreviewError, self)
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
