use crate::{State, path::Path};
use crate::{
    db::{categories_for_feed, rows_into_vec},
    inc_sql,
    model::json::{Feed, FeedEpisode},
    util::percent_decode,
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
    dbg!(&query.term);
    // let search_term = percent_decode(&query.term);
    // dbg!(&search_term);
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

//TODO maybe also by id ??
pub async fn by_name(path: Path<String>, state: web::Data<State>) -> ApiJsonResult<FeedEpisode> {
    // let feed_id = id
    //     .into_inner()
    //     .parse::<i32>()
    //     .map_err(|_err| ApiError::BadRequest)?;
    let feed_name = path.decode();
    let client = state.db_pool.get().await?;
    let feed_id_stmnt = client.prepare(inc_sql!("get/feed/id_for_name")).await?;
    let feed_id: i32 = client
        .query_one(&feed_id_stmnt, &[&feed_name])
        .await
        .map_err(|_e| ApiError::FeedByNameNotFound(feed_name.clone()))?
        .get("id");
    let feed_stmnt = client.prepare(inc_sql!("get/feed/by_name")).await?;
    let feed_row = client
        .query_one(&feed_stmnt, &[&feed_name])
        .await
        .map_err(|_e| ApiError::FeedByNameNotFound(feed_name.clone()))?;
    let episodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let episode_rows = client.query(&episodes_stmnt, &[&feed_id]).await?;
    let episodes = rows_into_vec(episode_rows);

    let categories = categories_for_feed(&client, feed_id).await?;
    let feed = FeedEpisode::from(&feed_row, categories, episodes).await?;
    Ok(Json(feed))
}

pub async fn by_category(
    state: web::Data<State>,
    category: Path<String>,
) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;

    let rows = if let Ok(category_id) = category.parse::<i32>() {
        let stmnt = client.prepare(inc_sql!("get/feed/by_category_id")).await?;
        client.query(&stmnt, &[&category_id]).await?
    } else {
        let category_name = &category.decode();
        let stmnt = client
            .prepare(inc_sql!("get/feed/by_category_name"))
            .await?;
        client.query(&stmnt, &[category_name]).await?
    };
    if rows.is_empty() {
        return Err(ApiError::CategoryNotFound(category.clone()));
    }

    let feeds = future::try_join_all(rows.into_iter().map(|row| Feed::from(&client, row))).await?;

    Ok(Json(feeds))
}
