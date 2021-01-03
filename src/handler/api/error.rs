use actix_web::http::StatusCode;
use actix_web::web::HttpResponse;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("other error: {0}")]
    Tokio(#[from] tokio_postgres::Error),
    #[error("pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("category {0} was not found")]
    CategoryNotFound(String),
    #[error("feed {0} was not found")]
    FeedNotFound(i32),
    #[error("author {0} was not found")]
    AuthorNotFound(i32),
}

#[derive(Debug, serde::Serialize)]
pub struct JsonError {
    error: String,
}
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::FeedNotFound(_)
            | ApiError::CategoryNotFound(_)
            | ApiError::AuthorNotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        HttpResponse::build(self.status_code()).json(JsonError {
            error: self.to_string(),
        })
    }
}
