use crate::{
    db::{api::fetch_feeds, api::fetch_feeds_by_name},
    model::Feed,
    State,
};

use crate::db::error::DbError;
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{web, web::Json};
use std::convert::From;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("DB error: {0}")]
    DB(#[from] DbError),
}

impl From<deadpool_postgres::PoolError> for ApiError {
    fn from(error: deadpool_postgres::PoolError) -> Self {
        Self::DB(DbError::Deadpool(error))
    }
}
#[derive(Debug, serde::Serialize)]
pub struct JsonError {
    error: String,
}
impl actix_web::ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        HttpResponseBuilder::new(self.status_code()).json(JsonError {
            error: self.to_string(),
        })
    }
}

pub async fn all_feeds(state: web::Data<State>) -> Result<Json<Vec<Feed>>, ApiError> {
    let mut client = state.db_pool.get().await?;
    Ok(Json(fetch_feeds(&mut client).await?))
}

pub async fn feeds_by_name(
    web::Path(title): web::Path<String>,
    state: web::Data<State>,
) -> Result<Json<Vec<Feed>>, ApiError> {
    let mut client = state.db_pool.get().await?;
    Ok(Json(fetch_feeds_by_name(&mut client, &title).await?))
}
