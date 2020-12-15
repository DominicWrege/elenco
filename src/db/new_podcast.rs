use std::collections::BTreeMap;

use crate::{
    inc_sql,
    model::{EpisodeRow, RawFeed},
};
use deadpool_postgres::Client;

use futures_util::future;
use tokio_postgres::Transaction;

pub async fn insert_feed<'a>(
    client: &mut Client,
    feed_content: &RawFeed<'a>,
    user_id: i32,
) -> Result<(), anyhow::Error> {
    let trx = client.transaction().await?;
    let autor_id = insert_or_get_author_id(&trx, feed_content.author).await;

    let language = if let Some(lang) = feed_content.language_code {
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

    let row = trx
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
    let new_feed_id: i32 = row.get("id");

    insert_feed_catagories(&trx, &feed_content.categories, new_feed_id).await?;
    insert_episodes(&trx, new_feed_id, &feed_content.episodes).await?;
    trx.commit().await?;
    Ok(())
}

pub async fn insert_or_get_language_id(
    trx: &Transaction<'_>,
    category: &str,
) -> Result<i32, tokio_postgres::Error> {
    let stmnt = trx.prepare(inc_sql!("insert/language")).await?;
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
    let stmnt = trx.prepare(inc_sql!("insert/episode")).await?;
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
