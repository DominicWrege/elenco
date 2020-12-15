use crate::{inc_sql, model::Feed};

use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::Client;

use super::rows_into_vec;
use crate::db::error::DbError;

pub async fn fetch_feeds(client: &mut Client) -> Result<Vec<Feed>, DbError> {
    let rows = client.query(inc_sql!("get/all_feeds"), &[]).await?;
    Ok(rows_into_vec(rows))
}

pub async fn fetch_feeds_by_name(client: &Client, name: &str) -> Result<Vec<Feed>, DbError> {
    let stmnt = client
        .prepare(
            "SELECT id, url, img_path, title, description, author_id FROM feed WHERE title LIKE concat('%', $1::text,'%') ORDER BY id",
        )
        .await?;
    let rows = client.query(&stmnt, &[&name.to_string()]).await?;
    Ok(rows_into_vec(rows))
}

pub async fn get_feeds_for_account(client: &Client, account_id: i32) -> Result<Vec<Feed>, DbError> {
    let stmnt = client.prepare(inc_sql!("get/feeds_for_account")).await?;
    let rows = client.query(&stmnt, &[&account_id]).await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| Feed::from_row(r).ok())
        .collect::<Vec<_>>())
}
