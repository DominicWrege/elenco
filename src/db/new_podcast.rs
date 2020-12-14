use std::collections::BTreeMap;

use crate::model::{EpisodeRow, RawFeed};
use deadpool_postgres::Client;

use futures_util::future;
use tokio_pg_mapper_derive::PostgresMapper;
use tokio_postgres::Transaction;

pub async fn insert_feed<'a>(
    client: &mut Client,
    feed_content: &RawFeed<'a>,
    user_id: i32,
) -> Result<(), anyhow::Error> {
    let trx = client.transaction().await?;
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

async fn insert_subcategories(
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

async fn insert_or_get_category_id(
    trx: &Transaction<'_>,
    category: &str,
    parent_id: Option<i32>,
) -> Result<i32, tokio_postgres::Error> {
    const WITHOUT_PARENT: &str = "
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
    ";
    const WITH_PARENT: &str = "
        WITH inserted as (
            INSERT INTO
            category(description, parent_id)
            VALUES
                ($1,$2)
            ON CONFLICT DO NOTHING
            RETURNING ID
        )
        SELECT id FROM inserted

        UNION ALL

        SELECT id
        FROM category
        WHERE description = $1
    ";

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
