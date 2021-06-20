use crate::{db::rows_into_vec, inc_sql, model::Author, path::Path};
use crate::{model::feed::Feed, State};
use actix_web::{web, web::Json};
use futures_util::future;

use super::{error::ApiError, ApiJsonResult};

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Author>> {
    let client = state.db_pool.get().await?;

    let rows = client.query(inc_sql!("get/author/all"), &[]).await?;
    let authors = rows_into_vec(rows);
    Ok(Json(authors))
}

pub async fn feeds(state: web::Data<State>, author_path: Path<String>) -> ApiJsonResult<Vec<Feed>> {
    let client = state.db_pool.get().await?;
    let author = author_path.decode();
    let rows = match author.parse::<i32>() {
        Ok(author_id) => {
            let stmnt = client.prepare(inc_sql!("get/feed/by_author_id")).await?;
            client.query(&stmnt, &[&author_id]).await?
        }
        Err(_) => {
            // dbg!(&author.replace(r#"\""#, ""));
            let stmnt = client.prepare(inc_sql!("get/feed/by_author_name")).await?;
            client.query(&stmnt, &[&author]).await?
        }
    };

    if rows.is_empty() {
        return Err(ApiError::AuthorNotFound(author));
    }
    let feeds = future::try_join_all(rows.into_iter().map(|row| Feed::from(&client, row))).await?;
    Ok(Json(feeds))
}
