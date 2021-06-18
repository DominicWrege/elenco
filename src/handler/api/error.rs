use actix_web::error::ErrorInternalServerError;
use actix_web::{body::Body, http::StatusCode, BaseHttpResponse};
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
    pub fn into_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        BaseHttpResponse::build(self.status_code)
            .content_type(mime::APPLICATION_JSON)
            .body(self.clone())
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

pub fn not_found() -> BaseHttpResponse<actix_web::dev::Body> {
    let json = JsonError {
        message: String::from("resource does not exist"),
        status_code: StatusCode::NOT_FOUND,
    };

    BaseHttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(mime::APPLICATION_JSON)
        .body(serde_json::to_string(&json).unwrap())
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::CategoryNotFound(_)
            | ApiError::FeedByIdNotFound(_)
            | ApiError::FeedByNameNotFound(_)
            | ApiError::AuthorNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::MissingTerm => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{}", self.to_string());
        crate::json_error!(ApiError, self)
    }
}

pub fn log_error<E: Into<anyhow::Error>>(err: E) -> actix_web::Error {
    let err = err.into();
    log::error!("{:?}", err);
    ErrorInternalServerError(err)
}
