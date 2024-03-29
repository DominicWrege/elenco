use actix_web::error::ErrorInternalServerError;
use actix_web::HttpResponse;
use actix_web::HttpResponseBuilder;
use actix_web::{body::Body, http::StatusCode};
use std::fmt::Display;

use thiserror::Error;

use crate::{generic_handler_err, hide_internal};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("internal error: {0:#?}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
    #[error("category {0} was not found")]
    CategoryNotFound(String),
    #[error("feed {0} was not found")]
    FeedByIdNotFound(i32),
    #[error("feed {0} was not found")]
    FeedByNameNotFound(String),
    #[error("author {0} was not found or has currently no online episodes")]
    AuthorNotFound(String),
    #[error("missing field `term`")]
    MissingTerm,
    #[error("{0}")]
    BadRequest(#[from] actix_web::Error),
    #[error("episode id: {0} not found")]
    EpisodeNotFound(i64),
    #[error("unauthorized access")]
    Unauthorized,
    #[error("User has no permission to access the moderator site")]
    Forbidden,
}
generic_handler_err!(ApiError, ApiError::Internal);

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JsonError {
    message: String,
    #[serde(with = "http_serde::status_code")]
    status_code: StatusCode,
}

impl JsonError {
    pub fn new(message: String, status_code: StatusCode) -> Self {
        Self {
            message,
            status_code,
        }
    }
    pub fn into_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code).json(self.clone())
    }
}

impl Into<actix_web::dev::Body> for JsonError {
    fn into(self) -> actix_web::dev::Body {
        Body::from(self.to_string())
    }
}

impl Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json_str = serde_json::to_string(&self).unwrap();
        write!(f, "{}", json_str)
    }
}

pub fn not_found() -> HttpResponse {
    let status_code = StatusCode::NOT_FOUND;
    let body = JsonError {
        message: String::from("resource does not exist"),
        status_code,
    };

    HttpResponseBuilder::new(status_code).json(body)
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden => StatusCode::FORBIDDEN,
            ApiError::CategoryNotFound(_)
            | ApiError::FeedByIdNotFound(_)
            | ApiError::FeedByNameNotFound(_)
            | ApiError::EpisodeNotFound(_)
            | ApiError::AuthorNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) | ApiError::MissingTerm => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self.to_string());
        crate::json_error!(ApiError, self)
    }
}

pub fn log_error<E: Into<anyhow::Error>>(err: E) -> actix_web::Error {
    let err = err.into();
    log::error!("{:?}", err);
    ErrorInternalServerError(err)
}
