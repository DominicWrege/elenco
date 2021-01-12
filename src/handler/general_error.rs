use crate::{
    generic_handler_err,
    template::{self},
};
use actix_web::{
    dev::{self, ServiceResponse},
    error::ErrorInternalServerError,
    middleware::ErrorHandlerResponse,
};
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;
#[derive(Error, Debug)]
#[error("{0:#?}")]
pub struct GeneralError(Box<dyn std::error::Error + Send + Sync + 'static>);

generic_handler_err!(GeneralError, GeneralError);

impl ResponseError for GeneralError {
    fn error_response(&self) -> HttpResponse {
        log::error!("{:#?}", self.to_string());
        HttpResponse::build(self.status_code())
            .content_type("text/html")
            .body(template::ErrorSite::html())
    }
}

pub async fn not_found(session: actix_session::Session) -> HttpResponse {
    template::NotFound::render_response(&session)
}

pub fn log_error<E: Into<anyhow::Error>>(err: E) -> actix_web::Error {
    let err = err.into();
    log::error!("{:?}", err);
    ErrorInternalServerError(err)
}

pub fn render_500<B>(
    res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let new_res = ServiceResponse::new(
        res.request().clone(),
        HttpResponse::InternalServerError()
            .body(template::ErrorSite::html())
            .into_body(),
    );

    Ok(ErrorHandlerResponse::Response(new_res))
}
