use crate::podcast::FeedContent;
use deadpool_postgres::Pool;
pub async fn save_feed<'a>(
    pool: &Pool,
    content: &FeedContent<'a>,
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
#[derive(Debug, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "feed")]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub img: Option<String>,
    pub title: String,
    pub description: String,
    pub author: String,
}

pub async fn fetch_feeds(pool: &Pool) -> Result<Vec<Feed>, anyhow::Error> {
    let client = pool.get().await?;

    let ret = client
        .query(
            "SELECT id, link as url, img_url as img, title, description, author FROM feed",
            &[],
        )
        .await?
        .into_iter()
        .filter_map(|r| Feed::from_row(r).ok())
        .collect::<Vec<_>>();
    Ok(ret)
}
