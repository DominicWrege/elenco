use deadpool_postgres::Client;

use super::api::DbError;

pub async fn is_moderator(client: &Client, id: i32) -> Result<bool, DbError> {
    let stmnt = client
        .prepare("SELECT id from Account WHERE id = $1 AND account_type = 'admin'")
        .await?;
    Ok(client.query_one(&stmnt, &[&id]).await.is_ok())
}
