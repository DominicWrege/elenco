use crate::{
    db::{api::fetch_feeds, api::fetch_feeds_by_name, api::DbError, new_podcast::SmallFeed},
    State,
};
use actix_web::{web, web::Json};

use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal error: {0}")]
    General(#[from] anyhow::Error),
    #[error("DB error: {0}")]
    DB(#[from] DbError),
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

type ApiFeedsResponse = Result<Json<Vec<SmallFeed>>, ApiError>;

pub async fn all_feeds(state: web::Data<State>) -> ApiFeedsResponse {
    Ok(Json(
        fetch_feeds(&mut state.db_pool.get().await.unwrap()).await?,
    ))
}

pub async fn feeds_by_name(
    web::Path(title): web::Path<String>,
    state: web::Data<State>,
) -> ApiFeedsResponse {
    Ok(Json(
        fetch_feeds_by_name(&state.db_pool.get().await.unwrap(), &title).await?,
    ))
}
