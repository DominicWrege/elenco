use crate::{
    inc_sql,
    model::{Comment, NewComment},
    Client,
};

pub async fn insert(
    client: &mut Client,
    comment: NewComment,
) -> Result<Comment, tokio_postgres::Error> {
    let stmnt = client.prepare(inc_sql!("insert/comment")).await?;
    let trx = client.transaction().await?;

    let row = trx
        .query_one(
            &stmnt,
            &[&comment.content, &comment.user_id, &comment.feed_id],
        )
        .await?;
    trx.commit().await?;
    Ok(Comment::from(row))
}

pub async fn get(client: &Client, feed_id: i32) -> Result<Vec<Comment>, tokio_postgres::Error> {
    let stmnt = client.prepare(inc_sql!("get/comment")).await?;
    let rows = client.query(&stmnt, &[&feed_id]).await?;

    let comments = rows
        .into_iter()
        .map(|row| Comment::from(row))
        .collect::<Vec<_>>();

    Ok(comments)
}
