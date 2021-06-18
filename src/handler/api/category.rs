use crate::model::category::Category;
use crate::State;
use crate::{inc_sql, path};
use actix_web::{web, web::Json};

use super::{error::ApiError, ApiJsonResult};

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Category>> {
    let client = state.db_pool.get().await?;
    let stmnt = inc_sql!("get/category/all");
    let rows = client.query(stmnt, &[]).await?;
    let categories = rows
        .iter()
        .map(|row| {
            Category::from(
                row,
                serde_json::from_value(row.get("subcategories")).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    Ok(Json(categories))
}

pub async fn by_id_or_name(
    state: web::Data<State>,
    path: path::Path<String>,
) -> ApiJsonResult<Category> {
    let client = state.db_pool.get().await?;
    let result = if let Ok(category_id) = path.parse::<i32>() {
        let stmnt = client.prepare(inc_sql!("get/category/by_id")).await?;
        client.query_one(&stmnt, &[&category_id]).await
    } else {
        let category_name = path.as_str();
        let stmnt = client.prepare(inc_sql!("get/category/by_name")).await?;
        client.query_one(&stmnt, &[&category_name]).await
    };
    let row = result.map_err(|_e| ApiError::CategoryNotFound(path.decode()))?;
    let catagories = Category::from(
        &row,
        serde_json::from_value(row.get("subcategories")).unwrap(),
    );
    Ok(Json(catagories))
}
