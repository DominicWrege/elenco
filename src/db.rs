use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::Row;

pub mod admin;
pub mod api;
pub mod error;
pub mod new_podcast;
pub mod util;

pub fn rows_into_vec<T>(row: Vec<Row>) -> Vec<T>
where
    T: FromTokioPostgresRow,
{
    row.into_iter()
        .filter_map(|r| T::from_row(r).ok())
        .collect::<Vec<_>>()
}
