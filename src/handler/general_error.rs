use crate::{
    generic_handler_err,
    template::{self},
};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use askama::Template;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0:#?}")]
pub struct GeneralError(Box<dyn std::error::Error + Send + Sync + 'static>);

generic_handler_err!(GeneralError, GeneralError);

impl ResponseError for GeneralError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        log::error!("{:#?}", self.to_string());
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

pub async fn not_found(session: actix_session::Session) -> HttpResponse {
    template::NotFound::render_response(&session)
}
