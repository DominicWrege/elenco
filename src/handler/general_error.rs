use crate::{generic_handler_err, handler::api::error};
use actix_web::{
    body::Body,
    dev::{self, ServiceResponse},
    error::ErrorInternalServerError,
    middleware::ErrorHandlerResponse,
    BaseHttpResponse,
};
use actix_web::{HttpResponse, ResponseError};
use error::JsonError;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("{0:#?}")]
pub struct GeneralError(Box<dyn std::error::Error + Send + Sync + 'static>);

generic_handler_err!(GeneralError, GeneralError);

const INTER_ERROR_MSG: &str = "Internal server error";

impl ResponseError for GeneralError {
    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{:#?}", self.to_string());
        JsonError::new(String::from(INTER_ERROR_MSG), self.status_code()).into_response()
    }
}

// fix me: merge with api-> error
pub async fn not_found() -> BaseHttpResponse<actix_web::dev::Body> {
    //template::NotFound::render_response(&session)
    error::not_found()
}

pub fn log_error<E: Into<anyhow::Error>>(err: E) -> actix_web::Error {
    let err = err.into();
    log::error!("{:?}", err);
    ErrorInternalServerError(err)
}
