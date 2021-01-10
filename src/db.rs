use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::{Client, Row};

use crate::{
    handler::{api::error::ApiError, general_error::GeneralError},
    inc_sql,
    model::json::Category,
};

pub mod preview_error;
pub mod save_feed;
pub mod util;

pub fn rows_into_vec<T>(row: Vec<Row>) -> Vec<T>
where
    T: FromTokioPostgresRow,
{
    row.into_iter()
        .filter_map(|r| T::from_row(r).ok())
        .collect::<Vec<_>>()
}

pub async fn feed_exits(
    client: &Client,
    title: &str,
    url: &str,
) -> Result<bool, tokio_postgres::Error> {
    let stmnt = client.prepare(inc_sql!("get/feed/exists")).await?;
    Ok(client.query_one(&stmnt, &[&title, &url]).await.is_ok())
}

pub async fn is_moderator(client: &Client, id: i32) -> Result<bool, GeneralError> {
    let stmnt = client
        .prepare("SELECT id from Account WHERE id = $1 AND account_type = 'admin'")
        .await?;
    Ok(client.query_one(&stmnt, &[&id]).await.is_ok())
}

pub async fn categories_for_feed(client: &Client, feed_id: i32) -> Result<Vec<Category>, ApiError> {
    let categories_stmnt = client.prepare(inc_sql!("get/category/by_feed_id")).await?;
    let categories_rows = client.query(&categories_stmnt, &[&feed_id]).await?;
    let mut categories = Vec::new();
    for row in &categories_rows {
        let subcategories_stmnt = client
            .prepare(inc_sql!("get/category/sub_by_feed_id"))
            .await?;
        let category_id: i32 = row.get("id");
        let subcategories_rows = client
            .query(&subcategories_stmnt, &[&feed_id, &category_id])
            .await?;
        let subcategories = rows_into_vec(subcategories_rows);
        categories.push(Category::from(row, subcategories));
    }

    Ok(categories)
}
