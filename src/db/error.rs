#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    Deadpool(#[from] deadpool_postgres::PoolError),
    #[error(transparent)]
    Postgres(#[from] tokio_postgres::Error),
}
