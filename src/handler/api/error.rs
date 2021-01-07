use actix_web::http::StatusCode;
use actix_web::web::HttpResponse;

use thiserror::Error;

use crate::{generic_handler_err, hide_internal};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("internal error: {0:#?}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
    #[error("category {0} was not found")]
    CategoryNotFound(String),
    #[error("feed {0} was not found")]
    FeedNotFound(i32),
    #[error("author {0} was not found")]
    AuthorNotFound(i32),
    #[error("missing field `term`")]
    MissingTerm,
}
generic_handler_err!(ApiError, ApiError::Internal);

#[derive(Debug, serde::Serialize)]
pub struct JsonError {
    error: String,
    status: u16,
}

pub async fn not_found() -> HttpResponse {
    let error = JsonError {
        error: String::from("resource does not exist"),
        status: StatusCode::NOT_FOUND.as_u16(),
    };
    HttpResponse::NotFound().json(error)
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::FeedNotFound(_)
            | ApiError::CategoryNotFound(_)
            | ApiError::AuthorNotFound(_) => StatusCode::NOT_FOUND,
            ApiError::MissingTerm => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        let status = self.status_code();
        let error_msg = hide_internal!(ApiError, self);

        HttpResponse::build(status).json(JsonError {
            error: error_msg,
            status: status.as_u16(),
        })
    }
}
