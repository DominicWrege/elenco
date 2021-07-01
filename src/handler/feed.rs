use crate::{
    db::{self, rows_into_vec},
    inc_sql,
    model::{
        episode::Episode,
        feed::{Feed, TinyFeed},
        user::Account,
        Completion, Permission,
    },
    util::percent_decode,
};
use crate::{path::Path, State};
use actix_session::Session;
use actix_web::{web, web::Json};

use futures_util::future;

use super::{error::ApiError, ApiJsonResult};

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;
    let feeds_row = client.query(inc_sql!("get/feed/all"), &[]).await?;
    let feeds = future::try_join_all(
        feeds_row
            .into_iter()
            .map(|row| Feed::from(&client, row, None)),
    )
    .await?;
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
    let search_term = percent_decode(&query.term);
    let feeds_row = match &query.lang {
        Some(lang) => {
            match &query.category {
                Some(category) => {
                    let feed_stmnt = client
                        .prepare(inc_sql!("get/feed/search/with_language_category"))
                        .await?;
                    client
                        .query(&feed_stmnt, &[&search_term, lang, category])
                        .await?
                }
                None => {
                    let feed_stmnt = client
                        .prepare(inc_sql!("get/feed/search/with_language"))
                        .await?;
                    client.query(&feed_stmnt, &[&search_term, lang]).await?
                }
            };
            let feed_stmnt = client
                .prepare(inc_sql!("get/feed/search/with_language"))
                .await?;
            client.query(&feed_stmnt, &[&search_term, lang]).await?
        }
        None => {
            let feed_stmnt = client.prepare(inc_sql!("get/feed/search/all")).await?;
            client.query(&feed_stmnt, &[&search_term]).await?
        }
    };
    let feeds = future::try_join_all(
        feeds_row
            .into_iter()
            .map(|row| Feed::from(&client, row, None)),
    )
    .await?;
    Ok(Json(feeds))
}

pub async fn by_name_or_id(
    path: Path<String>,
    state: web::Data<State>,
    session: Session,
) -> ApiJsonResult<Feed> {
    let client = state.db_pool.get().await?;

    let feed_id = match path.parse::<i32>() {
        Ok(id) => id,
        Err(_) => {
            let feed_name = path.decode();
            let feed_id_stmnt = client.prepare(inc_sql!("get/feed/id_for_name")).await?;
            client
                .query_one(&feed_id_stmnt, &[&feed_name])
                .await
                .map_err(|_e| ApiError::FeedByNameNotFound(feed_name.clone()))?
                .get("id")
        }
    };

    let feed_stmnt = match Account::from_session(&session) {
        Some(account) if account.permission() == Permission::Admin => {
            client.prepare(inc_sql!("get/feed/moderator/by_id")).await?
        }
        Some(account) => {
            let submitter_check_stmnt = client
                .prepare(inc_sql!("get/feed/user/submitter_check"))
                .await?;
            let x = client
                .query_one(&submitter_check_stmnt, &[&feed_id, &account.id()])
                .await;
            dbg!(&x);
            if x.is_ok() {
                client.prepare(inc_sql!("get/feed/moderator/by_id")).await?
            } else {
                client.prepare(inc_sql!("get/feed/by_id")).await?
            }
        }
        _ => client.prepare(inc_sql!("get/feed/by_id")).await?,
    };

    let feed_row = client
        .query_one(&feed_stmnt, &[&feed_id])
        .await
        .map_err(|_e| ApiError::FeedByIdNotFound(feed_id))?;

    let episodes_stmnt = client.prepare(inc_sql!("get/episodes_for_feed_id")).await?;
    let episode_rows = client.query(&episodes_stmnt, &[&feed_id]).await?;
    let episodes = episode_rows
        .into_iter()
        .map(|row| Episode::from(row))
        .collect::<Vec<_>>();

    let categories = db::category::get_categories_for_feed(&client, feed_id).await?;
    let feed = Feed::from(&client, feed_row, Some(episodes)).await?;
    Ok(Json(feed))
}

pub async fn completion(
    path: Path<String>,
    state: web::Data<State>,
) -> ApiJsonResult<Vec<Completion>> {
    let name = &path.decode();
    if name.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let client = state.db_pool.get().await?;
    let query_is_ok_stmnt = client.prepare(inc_sql!("query_is_ok")).await?;
    let code: i32 = client
        .query_one(&query_is_ok_stmnt, &[&name])
        .await?
        .get("code");

    if code == 0 {
        return Ok(Json(vec![]));
    }
    let stmnt = client.prepare(inc_sql!("get/completion")).await?;
    let rows = client.query(&stmnt, &[&name]).await?;
    let completions = rows_into_vec(rows);
    Ok(Json(completions))
}

pub async fn by_category(
    state: web::Data<State>,
    category: Path<String>,
) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;

    let rows = if let Ok(category_id) = category.parse::<i32>() {
        let stmnt_exists = client.prepare(inc_sql!("get/category/exist_by_id")).await?;
        client
            .query_one(&stmnt_exists, &[&category_id])
            .await
            .map_err(|_e| ApiError::CategoryNotFound(category.clone()))?;

        let stmnt = client.prepare(inc_sql!("get/feed/by_category_id")).await?;
        client.query(&stmnt, &[&category_id]).await?
    } else {
        let category_name = &category.decode();
        let stmnt_exists = client
            .prepare(inc_sql!("get/category/exist_by_name"))
            .await?;
        client
            .query_one(&stmnt_exists, &[&category_name])
            .await
            .map_err(|_e| ApiError::CategoryNotFound(category.clone()))?;
        let stmnt_feeds = client
            .prepare(inc_sql!("get/feed/by_category_name"))
            .await?;
        client.query(&stmnt_feeds, &[category_name]).await?
    };

    let feeds =
        future::try_join_all(rows.into_iter().map(|row| Feed::from(&client, row, None))).await?;

    Ok(Json(feeds))
}

pub async fn related(
    state: web::Data<State>,
    feed_id: Result<actix_web::web::Path<i32>, actix_web::Error>,
) -> ApiJsonResult<Vec<TinyFeed>> {
    let feed_id = feed_id
        .map_err(|err| ApiError::BadRequest(err))?
        .into_inner();

    let client = state.db_pool.get().await?;

    let stmnt_category_id = client
        .prepare(inc_sql!("get/category/get_id_by_feed_id"))
        .await?;
    let category_id = client
        .query_one(&stmnt_category_id, &[&feed_id])
        .await?
        .get::<_, i32>("category_id");

    let stmnt_feeds = client.prepare(inc_sql!("get/feed/related")).await?;
    let rows = client
        .query(&stmnt_feeds, &[&category_id, &feed_id])
        .await?;

    let feeds = rows_into_vec(rows);

    Ok(Json(feeds))
}
