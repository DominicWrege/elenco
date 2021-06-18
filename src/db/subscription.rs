use crate::{inc_sql, Client};

pub async fn subscribe(
    client: &mut Client,
    user_id: i32,
    feed_id: i32,
) -> Result<(), tokio_postgres::Error> {
    let stmnt = client.prepare(inc_sql!("insert/subscription")).await?;
    let trx = client.transaction().await?;
    trx.execute(&stmnt, &[&user_id, &feed_id]).await?;
    trx.commit().await?;
    Ok(())
}

pub async fn unsubscribe(
    client: &mut Client,
    user_id: i32,
    feed_id: i32,
) -> Result<(), tokio_postgres::Error> {
    let stmnt = client.prepare(inc_sql!("delete/subscription")).await?;
    let trx = client.transaction().await?;
    trx.execute(&stmnt, &[&user_id, &feed_id]).await?;
    trx.commit().await?;
    Ok(())
}

pub async fn user_has_subscription(
    client: &Client,
    user_id: i32,
    feed_id: i32,
) -> Result<bool, tokio_postgres::Error> {
    let stmnt = client
        .prepare(inc_sql!("get/user_has_subscription"))
        .await?;
    let result = client.query_one(&stmnt, &[&user_id, &feed_id]).await;

    Ok(result.is_ok())
}
