use crate::{inc_sql, model::preview::episode::Episode};
use futures_util::future;
use tokio_postgres::{Client, Transaction};

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
pub async fn episode_offset(
    client: &Client,
    episodes: &[Episode],
    feed_id: &i32,
) -> Result<Option<i64>, tokio_postgres::Error> {
    if 50 > episodes.len() {
        return Ok(None);
    }

    let max_stmnt = client
        .prepare(
            r#"
                SELECT max(episode.id) as max
                FROM episode 
                where feed_id = $1"#,
        )
        .await?;
    let max_episode_id = client
        .query_one(&max_stmnt, &[&feed_id])
        .await?
        .get::<_, i64>("max");
    let max_row_id = episodes.iter().map(|e| e.id).max();
    let ret = if matches!(max_row_id, Some(id) if id >= max_episode_id) {
        None
    } else {
        max_row_id
    };
    Ok(ret)
}
