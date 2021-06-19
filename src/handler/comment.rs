use actix_web::{
    web::{self, Json},
    HttpResponse,
};

use crate::{db, model::NewComment};

use super::api::error::ApiError;

// valid if content is not empty
pub async fn new(
    state: web::Data<crate::State>,
    comment_json: Result<actix_web::web::Json<NewComment>, actix_web::Error>,
) -> Result<HttpResponse, ApiError> {
    let comment_json = comment_json.map_err(|err| ApiError::BadRequest(err))?;
    let mut client = state.db_pool.get().await?;

    let new_comment = db::comment::insert(&mut client, comment_json.into_inner()).await?;

    Ok(HttpResponse::Ok().json(&new_comment))
}

pub async fn get_for_feed(
    state: web::Data<crate::State>,
    feed_id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let client = state.db_pool.get().await?;
    let comments = db::comment::get(&client, feed_id.into_inner()).await?;

    Ok(HttpResponse::Ok().json(comments))
}
