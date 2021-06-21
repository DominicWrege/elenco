use actix_web::web::Json;

use self::error::ApiError;

pub mod author;
pub mod category;
pub mod episode;
pub mod error;
pub mod feed;

pub type ApiJsonResult<T> = Result<Json<T>, ApiError>;
