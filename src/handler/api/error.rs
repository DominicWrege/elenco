use actix_web::{body::Body, http::StatusCode, BaseHttpResponse};

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
    #[error("author {0} was not found or has currently no online episodes")]
    AuthorNotFound(String),
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

pub async fn not_found() -> BaseHttpResponse<actix_web::dev::Body> {
    let json = JsonError {
        error: String::from("resource does not exist"),
        status: StatusCode::NOT_FOUND.as_u16(),
    };

    BaseHttpResponse::build(StatusCode::NOT_FOUND)
        .content_type(mime::APPLICATION_JSON)
        .body(serde_json::to_string(&json).unwrap())
}

impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{}", self.to_string());
        let status = self.status_code();

        let body = serde_json::to_string(&JsonError {
            error: hide_internal!(ApiError, self),
            status: status.as_u16(),
        })
        .unwrap();

        actix_web::BaseHttpResponse::build(status)
            .content_type(mime::APPLICATION_JSON)
            .body(Body::from(body))
    }
}
