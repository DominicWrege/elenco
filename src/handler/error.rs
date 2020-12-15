use askama::Template;
use thiserror::Error;

use crate::{
    generic_handler_err,
    template::{self},
};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

// #[derive(Debug, Error)]
// pub enum GeneralError {
//     #[error("Internal error")]
//     Internal(Box<dyn std::error::Error + Send + Sync + 'static>),
// }

#[derive(Debug, Error)]
#[error("Internal error")]
pub struct GeneralError(Box<dyn std::error::Error + Send + Sync + 'static>);

generic_handler_err!(GeneralError, GeneralError);

impl ResponseError for GeneralError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(
            template::ErrorSite {
                status_code: self.status_code(),
                permission: None,
            }
            .render()
            .unwrap(),
        )
    }
}
