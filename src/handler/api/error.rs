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
    #[error("bad request")]
    BadRequest,
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
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        log::error!("{}", self.to_string());
        let status = self.status_code();

        HttpResponse::build(status).json(JsonError {
            error: hide_internal!(ApiError, self),
            status: status.as_u16(),
        })
    }
}
