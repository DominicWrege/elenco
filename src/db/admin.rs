use deadpool_postgres::Client;

use crate::{inc_sql, model::Feed};

use super::{error::DbError, rows_into_vec};

pub async fn is_moderator(client: &Client, id: i32) -> Result<bool, DbError> {
    let stmnt = client
        .prepare("SELECT id from Account WHERE id = $1 AND account_type = 'admin'")
        .await?;
    Ok(client.query_one(&stmnt, &[&id]).await.is_ok())
}

pub async fn queued_feeds(client: &Client) -> Result<Vec<Feed>, DbError> {
    let rows = client.query(inc_sql!("get/queued_feeds"), &[]).await?;
    Ok(rows_into_vec(rows))
}

pub async fn reviewed_feed(client: &Client) -> Result<Vec<Feed>, DbError> {
    let rows = client
        .query(inc_sql!("get/last_reviewed_feeds"), &[])
        .await?;
    Ok(rows_into_vec(rows))
}

// pub async fn approve_feed(client: &Client, feed_id: i32) -> Result<(), DbError> {
//     let stmnt = client.prepare(inc_sql!("update/aprove_feed")).await?;
//     client.execute(&stmnt, &[&feed_id]).await?;
//     Ok(())
// }
