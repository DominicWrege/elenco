use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::{Client, Row};

use crate::{handler::error::GeneralError, inc_sql};

pub mod error;
pub mod new_feed;
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
    let stmnt = client.prepare(inc_sql!("get/feed_exists")).await?;
    Ok(client.query_one(&stmnt, &[&title, &url]).await.is_ok())
}

pub async fn is_moderator(client: &Client, id: i32) -> Result<bool, GeneralError> {
    let stmnt = client
        .prepare("SELECT id from Account WHERE id = $1 AND account_type = 'admin'")
        .await?;
    Ok(client.query_one(&stmnt, &[&id]).await.is_ok())
}
