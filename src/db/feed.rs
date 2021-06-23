use crate::handler::save_preview_feed::error::PreviewSaveError;
use crate::model::preview::feed::FeedPreview;
use crate::Client;
use crate::{img_cache::RowImg, inc_sql};
use futures_util::future;
use tokio_postgres::Transaction;

use super::{
    category::insert_feed_catagories, episode::insert_episodes, insert_or_get_author_id,
    insert_or_get_img_id, insert_or_get_language_id,
};

#[derive(Debug)]
struct Context<'a> {
    user: &'a i32,
    author: &'a Option<i32>,
    language: &'a Option<i32>,
    img: &'a Option<i32>,
    feed: &'a FeedPreview<'a>,
}

pub async fn save(
    client: &mut Client,
    feed_content: &FeedPreview<'_>,
    user_id: i32,
    img: Option<RowImg<'_>>,
) -> Result<i32, PreviewSaveError> {
    let trx = client.transaction().await?;
    let author_id = insert_or_get_author_id(&trx, feed_content.author).await;
    let language = if let Some(lang) = feed_content.language {
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
        author: &author_id,
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
    Ok(feed_id)
}

async fn insert_feed(
    trx: &Transaction<'_>,
    context: &Context<'_>,
) -> Result<i32, PreviewSaveError> {
    let stmnt = trx.prepare(inc_sql!("insert/feed")).await?;

    let row = trx
        .query_one(
            &stmnt,
            &[
                &context.user,
                context.author,
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
