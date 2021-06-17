use crate::State;
use crate::{inc_sql, model::json::Episode};
use actix_web::{web, web::Json};

use super::{error::ApiError, ApiJsonResult};

pub async fn by_feed_id(
    state: web::Data<State>,
    id: web::Path<i32>,
) -> ApiJsonResult<Vec<Episode>> {
    let client = state.db_pool.get().await?;
    let feed_id = id.into_inner();
    let episodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let episode_rows = client.query(&episodes_stmnt, &[&feed_id]).await?;
    if episode_rows.is_empty() {
        return Err(ApiError::FeedByIdNotFound(feed_id));
    }
    let episodes = episode_rows
        .into_iter()
        .map(|row| Episode::from(row))
        .collect::<Vec<_>>();

    Ok(Json(episodes))
}
