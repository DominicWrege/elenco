use std::error::Error as _;
use tokio_postgres::error::SqlState;

use crate::handler::feed_preview::PreviewError;

#[derive(thiserror::Error, Debug)]
pub enum Field {
    #[error("already")]
    Title,
    #[error("URL")]
    Url,
    #[error("img")]
    Img,
}
impl From<tokio_postgres::Error> for PreviewError {
    fn from(error: tokio_postgres::Error) -> Self {
        let source = error
            .source()
            .and_then(|src| src.downcast_ref::<tokio_postgres::error::DbError>());

        if let Some(db_error) = source {
            return match error.code() {
                Some(code) if code == &SqlState::UNIQUE_VIOLATION => match db_error.constraint() {
                    Some(field) if field == "title" => PreviewError::Duplicate(Field::Title),
                    Some(field) if field == "url" => PreviewError::Duplicate(Field::Url),
                    Some(field) if field == "img_path" => PreviewError::Duplicate(Field::Img),
                    _ => PreviewError::Internal(error.into()),
                },
                _ => PreviewError::Internal(error.into()),
            };
        }
        PreviewError::Internal(error.into())
    }
}

impl From<deadpool_postgres::PoolError> for PreviewError {
    fn from(e: deadpool_postgres::PoolError) -> Self {
        PreviewError::Internal(e.into())
    }
}

impl From<anyhow::Error> for PreviewError {
    fn from(e: anyhow::Error) -> Self {
        PreviewError::Internal(e.into())
    }
}
