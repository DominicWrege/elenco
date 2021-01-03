use crate::State;
use crate::{db::rows_into_vec, inc_sql, model::json::Author};
use actix_web::{web, web::Json};

use tokio_pg_mapper::FromTokioPostgresRow;

use super::{error::ApiError, ApiJsonResult};

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Author>> {
    let client = state.db_pool.get().await?;

    let rows = client.query(inc_sql!("get/author/all"), &[]).await?;
    let authors = rows_into_vec(rows);
    Ok(Json(authors))
}

pub async fn by_id(
    state: web::Data<State>,
    auhtor_id: web::Path<i32>,
) -> ApiJsonResult<Author> {
    let client = state.db_pool.get().await?;
    let auhtor_id = auhtor_id.into_inner();
    let stmnt = client.prepare(inc_sql!("get/author/by_id")).await?;
    let row = client
        .query_one(&stmnt, &[&auhtor_id])
        .await
        .map_err(|_e| ApiError::AuthorNotFound(auhtor_id))?;
    let author = Author::from_row(row).unwrap();
    Ok(Json(author))
}
