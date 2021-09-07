use crate::{
    inc_sql,
    model::{feed::FeedUserMeta, Status},
    Client,
};

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

pub async fn user_subscription_info(
    client: &Client,
    user_id: i32,
    feed_id: i32,
) -> Result<FeedUserMeta, tokio_postgres::Error> {
    let stmnt_subs = client
        .prepare(inc_sql!("get/user_has_subscription"))
        .await?;
    let stmnt_owner = client.prepare(inc_sql!("get/user_is_owner")).await?;
    let sub_result = client.query_one(&stmnt_subs, &[&user_id, &feed_id]).await;
    let owner_result = client.query_one(&stmnt_owner, &[&user_id, &feed_id]).await;

    Ok(FeedUserMeta {
        has_subscribed: sub_result.is_ok(),
        is_owner: owner_result.is_ok(),
        status: owner_result.map(|r| r.get::<_, Status>("status")).ok(),
    })
}
