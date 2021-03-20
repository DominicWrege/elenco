use crate::State;
use crate::{db::rows_into_vec, inc_sql, model::json::Episode};
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
        return Err(ApiError::FeedNotFound(feed_id));
    }
    let episodes = rows_into_vec(episode_rows);

    Ok(Json(episodes))
}
