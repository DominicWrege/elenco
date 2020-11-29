use std::collections::BTreeSet;

use crate::model::{EpisodeRow, RawFeed};
use deadpool_postgres::{Client, Manager, Pool};

use futures_util::future;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::{tls::NoTlsStream, Row, Socket, Transaction};

pub async fn insert_feed<'a>(
    client: &mut Client,
    feed_content: &RawFeed<'a>,
    user_id: i32,
) -> Result<(), anyhow::Error> {
    let trx = client.transaction().await?;
    // TODO insert categories

    let autor_id = insert_or_get_author_id(&trx, feed_content.author).await;

    let language = if let Some(lang) = feed_content.language {
        insert_or_get_language_id(&trx, lang).await.ok()
    } else {
        None
    };
    let stmnt = trx
        .prepare(
            "
                INSERT INTO feed(
                    submitter_id, author_id, title, img_path, 
                    description, subtitle, url, language, link_web
                )
                VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
        )
        .await?;

    let r = trx
        .query_one(
            &stmnt,
            &[
                &user_id,
                &autor_id,
                &feed_content.title,
                &feed_content.img_path(),
                &feed_content.description,
                &feed_content.subtitle,
                &feed_content.url(),
                &language,
                &feed_content.link_web(),
            ],
        )
        .await?;
    let new_feed_id: i32 = r.get("id");

    insert_feed_catagories(&trx, &feed_content.categories, new_feed_id).await?;
    insert_episodes(&trx, new_feed_id, &feed_content.episodes).await?;
    trx.commit().await?;
    Ok(())
}

#[derive(Debug, PostgresMapper, serde::Serialize)]
#[pg_mapper(table = "feed")]
pub struct SmallFeed {
    pub id: i32,
    pub url: String,
    pub img_path: Option<String>,
    pub title: String,
    pub description: String,
    pub author_id: i32,
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

pub async fn fetch_feeds(client: &mut Client) -> Result<Vec<SmallFeed>, DbError> {
    let rows = client
        .query(
            "SELECT id, url, img_path, title, description, author_id FROM feed ORDER BY id",
            &[],
        )
        .await?;
    Ok(rows_into_vec(rows))
}

pub async fn fetch_feeds_by_name(pool: &Pool, name: &str) -> Result<Vec<SmallFeed>, anyhow::Error> {
    let client = pool.get().await?;

    let stmnt = client
        .prepare(
            "SELECT id, url, img_path, title, description, author_id FROM feed WHERE title LIKE concat('%', $1::text,'%') ORDER BY id",
        )
        .await?;
    let rows = client.query(&stmnt, &[&name.to_string()]).await?;
    Ok(rows_into_vec(rows))
}

struct DBContext {
    client: tokio_postgres::Client,
    connection: tokio_postgres::Connection<Socket, NoTlsStream>,
    config: tokio_postgres::Config,
}

async fn connect_with_conf() -> Result<DBContext, anyhow::Error> {
    let mut config = tokio_postgres::Config::default();
    config
        .user("harra")
        .password("hund")
        .dbname("podcast")
        .host("127.0.0.1");
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    Ok(DBContext {
        client,
        connection,
        config,
    })
}

pub async fn insert_or_get_language_id(
    trx: &Transaction<'_>,
    category: &str,
) -> Result<i32, tokio_postgres::Error> {
    let stmnt = trx
        .prepare(
            "
            WITH inserted as (
                INSERT INTO
                feed_language(name)
                VALUES
                    ($1)
                ON CONFLICT DO NOTHING
                RETURNING ID
            )
            SELECT id FROM inserted
        
            UNION ALL
        
            SELECT id
            FROM feed_language
            WHERE name = $1
    ",
        )
        .await?;
    let row = trx.query_one(&stmnt, &[&category]).await?;
    Ok(row.get("id"))
}

async fn insert_feed_catagories(
    trx: &Transaction<'_>,
    categories: &BTreeSet<&str>,
    feed_id: i32,
) -> Result<(), tokio_postgres::Error> {
    let category_ids = future::join_all(
        categories
            .into_iter()
            .map(|c| insert_or_get_category_id(&trx, c)),
    )
    .await
    .into_iter()
    .filter_map(|r| r.ok())
    .collect::<Vec<_>>();

    let stmnt = trx
        .prepare("INSERT INTO feed_category (feed_id, category_id) VALUES($1, $2)")
        .await?;

    for c_id in category_ids {
        trx.execute(&stmnt, &[&feed_id, &c_id]).await?;
    }

    Ok(())
}

async fn insert_or_get_author_id(trx: &Transaction<'_>, author_name: Option<&str>) -> Option<i32> {
    if let Some(name) = author_name {
        let stmnt = trx
            .prepare(
                "
            WITH inserted as (
                INSERT INTO
                    author(name)
                VALUES
                    ($1)
                ON CONFLICT DO NOTHING
                RETURNING ID
            )
            SELECT id FROM inserted
        
            UNION ALL
        
            SELECT id
            FROM author
            WHERE name = $1
    ",
            )
            .await
            .ok();
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

async fn insert_episodes(
    trx: &Transaction<'_>,
    feed_id: i32,
    episodes: &Vec<EpisodeRow<'_>>,
) -> Result<(), tokio_postgres::Error> {
    future::try_join_all(
        episodes
            .iter()
            .map(|ep| insert_one_episode(trx, feed_id, ep)),
    )
    .await?;

    Ok(())
}

async fn insert_one_episode(
    trx: &Transaction<'_>,
    feed_id: i32,
    ep: &EpisodeRow<'_>,
) -> Result<(), tokio_postgres::Error> {
    let stmnt = trx
        .prepare(
            "
        INSERT INTO  
            episode(title, description, published, explicit, keywords, 
                    duration, show_notes, url, media_url, feed_id )
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)

    ",
        )
        .await?;
    trx.execute(
        &stmnt,
        &[
            &ep.title,
            &ep.description,
            &ep.published,
            &ep.explicit,
            &ep.keywords,
            &ep.duration,
            &ep.show_notes,
            &ep.url(),
            &ep.media_url(),
            &feed_id,
        ],
    )
    .await?;

    Ok(())
}

async fn insert_or_get_category_id(
    trx: &Transaction<'_>,
    category: &str,
) -> Result<i32, tokio_postgres::Error> {
    dbg!(&category);
    let stmnt = trx
        .prepare(
            "
            WITH inserted as (
                INSERT INTO
                category(description)
                VALUES
                    ($1)
                ON CONFLICT DO NOTHING
                RETURNING ID
            )
            SELECT id FROM inserted
        
            UNION ALL
        
            SELECT id
            FROM category
            WHERE description = $1
    ",
        )
        .await?;
    let row = trx.query_one(&stmnt, &[&category]).await?;
    Ok(row.get("id"))
}

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub async fn connect_and_migrate() -> Result<Pool, anyhow::Error> {
    let DBContext {
        mut client,
        connection,
        config,
    } = connect_with_conf().await?;
    tokio::task::spawn(connection);
    embedded::migrations::runner()
        .run_async(&mut client)
        .await?;
    let mngr = Manager::new(config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, 12))
}
