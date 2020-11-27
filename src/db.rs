use crate::podcast::PreviewFeedContent;
use deadpool_postgres::{Client, Manager, Pool};
pub async fn save_feed<'a>(
    pool: &Pool,
    content: &PreviewFeedContent<'a>,
    user_id: i32,
) -> Result<(), anyhow::Error> {
    let mut client = pool.get().await?;

    let trx = client.transaction().await?;
    let stmnt = trx
        .prepare(
            "
                    INSERT INTO feed(account, title, img_url, description, link, author)
                    VALUES($1, $2, $3, $4, $5, $6)",
        )
        .await?;

    trx.execute(
        &stmnt,
        &[
            &user_id,
            &content.title,
            &content.img.as_ref().and_then(|o| Some(o.to_string())),
            &content.description,
            &content.url.to_string(),
            &content.author,
        ],
    )
    .await?;
    trx.commit().await?;
    Ok(())
}

use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Row;
#[derive(Debug, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "feed")]
pub struct SmallFeed {
    pub id: i32,
    pub url: String,
    pub img_path: Option<String>,
    pub title: String,
    pub description: String,
    pub author: String,
}

pub fn rows_into_vec<T>(row: Vec<Row>) -> Vec<T>
where
    T: FromTokioPostgresRow,
{
    row.into_iter()
        .filter_map(|r| T::from_row(r).ok())
        .collect::<Vec<_>>()
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Deadpool: {0}")]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error("tokio_postgres: {0}")]
    Postgres(#[from] tokio_postgres::Error),
}

pub async fn fetch_feeds(pool: &Pool) -> Result<Vec<SmallFeed>, DbError> {
    let client = pool.get().await?;

    let rows = client
        .query(
            "SELECT id, url, img_path, title, description, author FROM feed ORDER BY id",
            &[],
        )
        .await?;
    Ok(rows_into_vec(rows))
}

pub async fn fetch_feeds_by_name(pool: &Pool, name: &str) -> Result<Vec<SmallFeed>, anyhow::Error> {
    let client = pool.get().await?;

    let stmnt = client
        .prepare(
            "SELECT id, url, img_path, title, description, author FROM feed WHERE title LIKE concat('%', $1::text,'%') ORDER BY id",
        )
        .await?;
    let rows = client.query(&stmnt, &[&name.to_string()]).await?;
    Ok(rows_into_vec(rows))
}

// #[cfg(test)]
// mod tests {
//     // Note this useful idiom: importing names from outer (for mod tests) scope.
//     use super::*;

//     #[test]
//     fn test_insert_feed() {
//         dbg!("dhsazidh");
//         aa();
//         assert_eq!(3, 3);
//     }
// }

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

async fn connect_with_conf(
) -> Result<(tokio_postgres::Client, tokio_postgres::Config), anyhow::Error> {
    let mut pg_config = tokio_postgres::Config::default();
    pg_config
        .user("harra")
        .password("hund")
        .dbname("podcast")
        .host("127.0.0.1");
    Ok((pg_config.connect(tokio_postgres::NoTls).await?.0, pg_config))
}

pub async fn connect_and_migrate() -> Result<Pool, anyhow::Error> {
    let (mut client, pg_config) = connect_with_conf().await?;
    embedded::migrations::runner()
        .run_async(&mut client)
        .await?;
    let mngr = Manager::new(pg_config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, 12))
}
