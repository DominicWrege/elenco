use crate::model::FeedSmall2;

use thiserror::Error;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::{Client, Row};

use super::new_podcast::SmallFeed;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Deadpool: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("tokio_postgres: {0}")]
    Postgres(#[from] tokio_postgres::Error),
}

pub fn rows_into_vec<T>(row: Vec<Row>) -> Vec<T>
where
    T: FromTokioPostgresRow,
{
    row.into_iter()
        .filter_map(|r| T::from_row(r).ok())
        .collect::<Vec<_>>()
}

pub async fn fetch_feeds(client: &mut Client) -> Result<Vec<SmallFeed>, DbError> {
    let rows = client
        .query(
            "SELECT id, url, img_path, title, description, author_id FROM feed ORDER BY id",
            &[],
        )
        .await?;
    Ok(rows_into_vec(rows))
}

pub async fn fetch_feeds_by_name(
    client: &Client,
    name: &str,
) -> Result<Vec<SmallFeed>, anyhow::Error> {
    let stmnt = client
        .prepare(
            "SELECT id, url, img_path, title, description, author_id FROM feed WHERE title LIKE concat('%', $1::text,'%') ORDER BY id",
        )
        .await?;
    let rows = client.query(&stmnt, &[&name.to_string()]).await?;
    Ok(rows_into_vec(rows))
}

pub async fn get_feeds_for_account(
    client: &Client,
    account_id: i32,
) -> Result<Vec<FeedSmall2>, anyhow::Error> {
    let stmnt = client
        .prepare(
            "
            SELECT title, img_path, author.name as author_name, link_web, status::text
            FROM feed INNER JOIN author ON Feed.author_id = author.id
            WHERE feed.submitter_id = $1
            ",
        )
        .await?;
    let rows = client.query(&stmnt, &[&account_id]).await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| FeedSmall2::from_row(r).ok())
        .collect::<Vec<_>>())
}
