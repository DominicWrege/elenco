use std::collections::BTreeMap;

use futures_util::future;
use tokio_postgres::Transaction;

use crate::{handler::api::error::ApiError, inc_sql, model::category::Category, Client};

use super::rows_into_vec;

pub async fn insert_subcategories(
    trx: &Transaction<'_>,
    parent_category: i32,
    subcategories: &[&str],
) -> Result<Vec<i32>, tokio_postgres::Error> {
    future::try_join_all(
        subcategories
            .iter()
            .map(|child| insert_or_get_category_id(trx, child, Some(parent_category))),
    )
    .await
}

pub async fn insert_or_get_category_id(
    trx: &Transaction<'_>,
    category: &str,
    parent_id: Option<i32>,
) -> Result<i32, tokio_postgres::Error> {
    const WITHOUT_PARENT: &str = inc_sql!("insert/category");
    const WITH_PARENT: &str = inc_sql!("insert/category_with_parent");
    let stmnt = trx
        .prepare(if parent_id.is_some() {
            WITH_PARENT
        } else {
            WITHOUT_PARENT
        })
        .await?;

    let row = match parent_id {
        Some(parent_id) => trx.query_one(&stmnt, &[&category, &parent_id]).await?,
        None => trx.query_one(&stmnt, &[&category]).await?,
    };
    Ok(row.get("id"))
}

pub async fn insert_feed_catagories(
    trx: &Transaction<'_>,
    categories: &BTreeMap<&str, Vec<&str>>,
    feed_id: i32,
) -> Result<(), tokio_postgres::Error> {
    let stmnt = trx
        .prepare("INSERT INTO feed_category (feed_id, category_id) VALUES($1, $2)")
        .await?;

    for (parent, children) in categories {
        let parent_id = insert_or_get_category_id(trx, parent, None).await?;
        trx.execute(&stmnt, &[&feed_id, &parent_id]).await?;
        for child_id in insert_subcategories(trx, parent_id, children).await? {
            trx.execute(&stmnt, &[&feed_id, &child_id]).await?;
        }
    }

    Ok(())
}

pub async fn get_categories_for_feed(
    client: &Client,
    feed_id: i32,
) -> Result<Vec<Category>, ApiError> {
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
