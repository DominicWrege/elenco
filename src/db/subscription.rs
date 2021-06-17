use actix_web::HttpResponse;
use tokio_postgres::GenericClient;

use crate::{Client, handler::api::error::ApiError, inc_sql};

pub async fn subscribe(client: &Client, user_id: i32, feed_id: i32) -> Result<(), tokio_postgres::Error>{
    let stmnt = client.prepare(inc_sql!("insert/subscription")).await?;
    client.execute(&stmnt, &[&user_id, &feed_id]).await?;
    Ok(())
}

pub async fn unsubscribe(client: &Client, user_id: i32, feed_id: i32) -> Result<(), tokio_postgres::Error>{
    let stmnt = client.prepare(inc_sql!("delete/subscription")).await?;
    client.execute(&stmnt, &[&user_id, &feed_id]).await?;
    Ok(())
}

pub async fn user_has_subscription(client: &Client, user_id: i32, feed_id: i32) -> Result<bool, tokio_postgres::Error>{

    let stmnt = client.prepare(inc_sql!("get/user_has_subscription")).await?;
    let result = client.query_one(&stmnt, &[&user_id, &feed_id]).await;

    Ok(result.is_ok())

}