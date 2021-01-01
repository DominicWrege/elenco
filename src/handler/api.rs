use crate::{
    db::rows_into_vec,
    inc_sql,
    model::{api::FeedJson, feed::Feed},
};
use crate::{model::api::Category, State};
use actix_web::http::StatusCode;
use actix_web::{dev::HttpResponseBuilder, HttpResponse};
use actix_web::{web, web::Json};

use thiserror::Error;
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("other error: {0}")]
    Tokio(#[from] tokio_postgres::Error),
    #[error("pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),
    #[error("category {0} was found")]
    CategoryNotFound(String),
    #[error("feed {0} was found")]
    FeedNotFound(i32),
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

pub async fn feeds_by(
    path: web::Path<i32>,
    state: web::Data<State>,
) -> Result<HttpResponse, ApiError> {
    let feed_id = path.into_inner();
    let client = state.db_pool.get().await?;
    let feed_stmnt = client.prepare(inc_sql!("get/online_feed_by_id")).await?;
    let categories_stmnt = client.prepare(inc_sql!("get/categories_by_feed")).await?;
    let feed_row = client
        .query_one(&feed_stmnt, &[&feed_id])
        .await
        .map_err(|_e| ApiError::FeedNotFound(feed_id))?;
    let categories = client
        .query(&categories_stmnt, &[&feed_id])
        .await?
        .into_iter()
        .map(|row| row.into())
        .collect::<Vec<Category>>();

    Ok(HttpResponse::Ok().json(FeedJson::from(&feed_row, &categories)))
}

pub async fn all_categories(state: web::Data<State>) -> Result<Json<Vec<Category>>, ApiError> {
    let client = state.db_pool.get().await?;

    let stmnt = inc_sql!("get/all_categories");

    let rows = client.query(stmnt, &[]).await?;
    let categories = rows.into_iter().map(|row| row.into()).collect::<Vec<_>>();

    Ok(Json(categories))
}

pub async fn category_by(
    state: web::Data<State>,
    path: web::Path<String>,
) -> Result<Json<Category>, ApiError> {
    let client = state.db_pool.get().await?;
    let result = if let Ok(category_id) = path.parse::<i32>() {
        let stmnt = client.prepare(inc_sql!("get/category_by_id")).await?;
        client.query_one(&stmnt, &[&category_id]).await
    } else {
        let category_name = path.as_str();
        let stmnt = client.prepare(inc_sql!("get/category_by_name")).await?;
        client.query_one(&stmnt, &[&category_name]).await
    };
    let row = result.map_err(|_e| ApiError::CategoryNotFound(path.into_inner()))?;

    Ok(Json(row.into()))
}
