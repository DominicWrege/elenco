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
