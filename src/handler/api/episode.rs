use crate::State;
use crate::{db::rows_into_vec, inc_sql, model::json::Epsiode};
use actix_web::{web, web::Json};

use super::{error::ApiError, ApiJsonResult};

pub async fn by_feed_id(
    state: web::Data<State>,
    id: web::Path<i32>,
) -> ApiJsonResult<Vec<Epsiode>> {
    let client = state.db_pool.get().await?;
    let feed_id = id.into_inner();
    let epsiodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let epsiode_rows = client.query(&epsiodes_stmnt, &[&feed_id]).await?;
    if epsiode_rows.is_empty() {
        return Err(ApiError::FeedNotFound(feed_id));
    }
    let epsiodes = rows_into_vec(epsiode_rows);

    Ok(Json(epsiodes))
}
