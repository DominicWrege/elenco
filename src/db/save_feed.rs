use std::collections::BTreeMap;

use crate::model::item::Episode;
use crate::{handler::preview_error::PreviewError, model::channel::Feed};
use crate::{img_cache::RowImg, inc_sql};
use deadpool_postgres::Client;
use futures_util::future;
use tokio_postgres::Transaction;

#[derive(Debug)]
struct Context<'a> {
    user: &'a i32,
    autor: &'a Option<i32>,
    language: &'a Option<i32>,
    img: &'a Option<i32>,
    feed: &'a Feed<'a>,
}

pub async fn save(
    client: &mut Client,
    feed_content: &Feed<'_>,
    user_id: i32,
    img: Option<RowImg<'_>>,
) -> Result<(), PreviewError> {
    let trx = client.transaction().await?;
    let autor_id = insert_or_get_author_id(&trx, feed_content.author).await;
    let language = if let Some(lang) = feed_content.language_code {
        insert_or_get_language_id(&trx, lang).await.ok()
    } else {
        None
    };

    let img_id: Option<i32> = if let Some(img) = &img {
        insert_or_get_img_id(&trx, img).await.ok()
    } else {
        None
    };

    let context = Context {
        user: &user_id,
        autor: &autor_id,
        language: &language,
        img: &img_id,
        feed: feed_content,
    };
    let feed_id = insert_feed(&trx, &context).await?;
    future::try_join(
        insert_feed_catagories(&trx, &feed_content.categories, feed_id),
        insert_episodes(&trx, feed_id, &feed_content.episodes),
    )
    .await?;
    trx.commit().await?;
    Ok(())
}

async fn insert_feed(trx: &Transaction<'_>, context: &Context<'_>) -> Result<i32, PreviewError> {
    let stmnt = trx.prepare(inc_sql!("insert/feed")).await?;

    let row = trx
        .query_one(
            &stmnt,
            &[
                &context.user,
                context.autor,
                &context.feed.title,
                &context.feed.description,
                context.img,
                &context.feed.subtitle,
                &context.feed.url(),
                context.language,
                &context.feed.link_web(),
            ],
        )
        .await?;
    Ok(row.get("id"))
}

async fn insert_or_get_img_id(
    trx: &Transaction<'_>,
    img: &RowImg<'_>,
) -> Result<i32, PreviewError> {
    let stmnt = trx.prepare(inc_sql!("insert/img")).await?;

    let row = trx
        .query_one(
            &stmnt,
            &[&img.link.clone().as_str(), &img.hash, &img.filename],
        )
        .await?;

    Ok(row.get("id"))
}

async fn insert_or_get_language_id(
    trx: &Transaction<'_>,
    language: &str,
) -> Result<i32, tokio_postgres::Error> {
    let stmnt = trx.prepare(inc_sql!("insert/language")).await?;
    let row = trx.query_one(&stmnt, &[&language]).await?;
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
    episodes: &[Episode<'_>],
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
    ep: &Episode<'_>,
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
