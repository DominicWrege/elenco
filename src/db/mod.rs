pub mod category;
pub mod episode;
pub mod feed;
pub mod subscription;

pub mod util;
use crate::{handler::general_error::GeneralError, inc_sql};
use crate::{handler::save_preview_feed::error::PreviewSaveError, img_cache::RowImg, Client};
use deadpool_postgres::Transaction;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::Row;

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

async fn insert_or_get_img_id(
    trx: &Transaction<'_>,
    img: &RowImg<'_>,
) -> Result<i32, PreviewSaveError> {
    let stmnt = trx.prepare(inc_sql!("insert/img")).await?;

    let row = trx
        .query_one(
            &stmnt,
            &[&img.link.clone().as_str(), &img.hash, &img.file_name],
        )
        .await?;

    Ok(row.get("id"))
}

pub async fn insert_or_get_language_id(
    trx: &Transaction<'_>,
    language: &str,
) -> Result<i32, tokio_postgres::Error> {
    let stmnt = trx.prepare(inc_sql!("insert/language")).await?;
    let row = trx.query_one(&stmnt, &[&language]).await?;
    Ok(row.get("id"))
}

pub async fn insert_or_get_author_id(
    trx: &Transaction<'_>,
    author_name: Option<&str>,
) -> Option<i32> {
    if let Some(name) = author_name {
        let stmnt = trx.prepare(inc_sql!("insert/author")).await.ok();
        if let Some(s) = stmnt {
            return trx
                .query_one(&s, &[&name])
                .await
                .ok()
                .and_then(|r| r.get("id"));
        }
    }
    None
}
