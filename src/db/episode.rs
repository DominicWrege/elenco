use crate::{inc_sql, model::preview::episode::Episode};
use futures_util::future;
use tokio_postgres::Transaction;

pub async fn insert_episodes(
    trx: &Transaction<'_>,
    feed_id: i32,
    episodes: &[Episode],
) -> Result<(), tokio_postgres::Error> {
    future::try_join_all(
        episodes
            .iter()
            .map(|ep| insert_one_episode(trx, feed_id, ep)),
    )
    .await?;

    Ok(())
}

pub async fn insert_one_episode(
    trx: &Transaction<'_>,
    feed_id: i32,
    ep: &Episode,
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
            &ep.guid,
            &ep.enclosure.length,
            &ep.enclosure.mime_type.to_string(),
        ],
    )
    .await?;

    Ok(())
}
