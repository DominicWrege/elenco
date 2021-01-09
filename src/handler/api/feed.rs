use crate::State;
use crate::{
    db::{categories_for_feed, rows_into_vec},
    inc_sql,
    model::json::{Feed, FeedEpisode},
};
use actix_web::{web, web::Json};

use futures_util::future;

use super::{error::ApiError, ApiJsonResult};

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;
    let feeds_row = client.query(inc_sql!("get/feed/all"), &[]).await?;
    let feeds =
        future::try_join_all(feeds_row.into_iter().map(|row| Feed::from(&client, row))).await?;
    Ok(Json(feeds))
}

#[derive(serde::Deserialize)]
pub struct SearchQuery {
    term: String,
    lang: Option<String>,
    category: Option<i32>,
}

pub async fn search(
    query: Result<web::Query<SearchQuery>, actix_web::Error>,
    state: web::Data<State>,
) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;
    let query = query.map_err(|_e| ApiError::MissingTerm)?;
    let feeds_row = match &query.lang {
        Some(lang) => {
            match &query.category {
                Some(category) => {
                    let feed_stmnt = client
                        .prepare(inc_sql!("get/feed/search_with_language_category"))
                        .await?;
                    client
                        .query(&feed_stmnt, &[&query.term, lang, category])
                        .await?
                }
                None => {
                    let feed_stmnt = client
                        .prepare(inc_sql!("get/feed/search_with_language"))
                        .await?;
                    client.query(&feed_stmnt, &[&query.term, lang]).await?
                }
            };
            let feed_stmnt = client
                .prepare(inc_sql!("get/feed/search_with_language"))
                .await?;
            client.query(&feed_stmnt, &[&query.term, lang]).await?
        }
        None => {
            let feed_stmnt = client.prepare(inc_sql!("get/feed/search")).await?;
            client.query(&feed_stmnt, &[&query.term]).await?
        }
    };
    let feeds =
        future::try_join_all(feeds_row.into_iter().map(|row| Feed::from(&client, row))).await?;
    Ok(Json(feeds))
}

pub async fn by_name_or_id(
    path: web::Path<i32>,
    state: web::Data<State>,
) -> ApiJsonResult<FeedEpisode> {
    let feed_id = path.into_inner();
    let client = state.db_pool.get().await?;
    let feed_stmnt = client.prepare(inc_sql!("get/feed/by_id")).await?;
    let feed_row = client
        .query_one(&feed_stmnt, &[&feed_id])
        .await
        .map_err(|_e| ApiError::FeedNotFound(feed_id))?;
    let categories = categories_for_feed(&client, feed_row.get("id")).await?;

    let epsiodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let epsiode_rows = client.query(&epsiodes_stmnt, &[&feed_id]).await?;
    let epsiodes = rows_into_vec(epsiode_rows);
    let feed = FeedEpisode::from(&feed_row, categories, epsiodes).await?;
    Ok(Json(feed))
}

pub async fn by_category(
    state: web::Data<State>,
    category: web::Path<String>,
) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;

    let rows = if let Ok(category_id) = category.parse::<i32>() {
        let stmnt = client.prepare(inc_sql!("get/feed/by_category_id")).await?;
        client.query(&stmnt, &[&category_id]).await?
    } else {
        let category_name = category.as_str();
        let stmnt = client
            .prepare(inc_sql!("get/feed/by_category_name"))
            .await?;
        client.query(&stmnt, &[&category_name]).await?
    };

    if rows.is_empty() {
        return Err(ApiError::CategoryNotFound(category.into_inner()));
    }

    let feeds = future::try_join_all(rows.into_iter().map(|row| Feed::from(&client, row))).await?;

    Ok(Json(feeds))
}

pub async fn by_author(
    state: web::Data<State>,
    auhtor_id: web::Path<i32>,
) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;
    let auhtor_id = auhtor_id.into_inner();
    let stmnt = client.prepare(inc_sql!("get/feed/by_author")).await?;
    let rows = client.query(&stmnt, &[&auhtor_id]).await?;
    if rows.is_empty() {
        return Err(ApiError::AuthorNotFound(auhtor_id));
    }
    let feeds = future::try_join_all(rows.into_iter().map(|row| Feed::from(&client, row))).await?;
    Ok(Json(feeds))
}
