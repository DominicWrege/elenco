use crate::{db::rows_into_vec, inc_sql, model::feed::Feed};
use crate::{model::api::Category, State};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::StatusCode;
use actix_web::{web, web::Json};
use serde_json;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Other error: {0}")]
    Tokio(#[from] tokio_postgres::Error),
    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
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
    let client = state.db_pool.get().await?;
    let rows = client.query(inc_sql!("get/all_feeds"), &[]).await?;
    Ok(Json(rows_into_vec(rows)))
}

pub async fn feeds_by_name(
    web::Path(title): web::Path<String>,
    state: web::Data<State>,
) -> Result<Json<Vec<Feed>>, ApiError> {
    let client = state.db_pool.get().await?;
    let stmnt = client
    .prepare(
        "select * from AllFeeds WHERE status = 'online' AND title LIKE concat('%', $1::text,'%') ORDER BY id",
    )
    .await?;
    let rows = client.query(&stmnt, &[&title.to_string()]).await?;
    Ok(Json(rows_into_vec(rows)))
}

pub async fn all_categories(state: web::Data<State>) -> Result<Json<Vec<Category>>, ApiError> {
    let client = state.db_pool.get().await?;

    let stmnt = inc_sql!("get/all_categories");

    let rows = client.query(stmnt, &[]).await?;
    let categories = rows.into_iter().map(|row| row.into()).collect::<Vec<_>>();

    Ok(Json(categories))
}

pub async fn category_by_id(
    state: web::Data<State>,
    path: web::Path<i32>,
) -> Result<Json<Category>, ApiError> {
    let client = state.db_pool.get().await?;
    let category_id = path.into_inner();
    let stmnt = client.prepare(inc_sql!("get/category_by_id")).await?;
    let row = client.query_one(&stmnt, &[&category_id]).await?;

    Ok(Json(row.into()))
}
