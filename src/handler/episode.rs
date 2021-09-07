use super::{error::ApiError, ApiJsonResult};
use crate::db;
use crate::inc_sql;
use crate::model::preview::episode::Episode;
use crate::model::preview::episode::EpisodeNext;
use crate::util::serialize;
use crate::State;
use actix_web::web;

#[derive(Debug, serde::Deserialize)]
pub struct QueryOffset {
    offset: Option<i64>,
    limit: Option<i64>,
}

pub async fn by_feed_id(
    state: web::Data<State>,
    id: web::Path<i32>,
    query: web::Query<QueryOffset>,
) -> ApiJsonResult<EpisodeNext> {
    let offset = query.offset.unwrap_or(-1);
    let limit = query.limit.unwrap_or(50);
    let client = state.db_pool.get().await?;
    let feed_id = id.into_inner();

    let episodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let episode_rows = client
        .query(&episodes_stmnt, &[&feed_id, &offset, &limit])
        .await?;

    if episode_rows.is_empty() {
        return Err(ApiError::FeedByIdNotFound(feed_id));
    }

    let episodes = episode_rows
        .into_iter()
        .map(|row| Episode::from(row))
        .collect::<Vec<_>>();

    serialize(EpisodeNext {
        offset: db::episode::episode_offset(&client, &episodes, &feed_id).await?,
        items: episodes,
    })
}

pub async fn by_episode_id(state: web::Data<State>, id: web::Path<i64>) -> ApiJsonResult<Episode> {
    let client = state.db_pool.get().await?;
    let id = id.into_inner();
    let stmnt = client.prepare(inc_sql!("get/episode")).await?;

    let row = client
        .query_one(&stmnt, &[&id])
        .await
        .map_err(|_err| ApiError::EpisodeNotFound(id))?;

    serialize(Episode::from(row))
}
