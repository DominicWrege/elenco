use crate::generic_handler_err;
use actix_web::{
    dev::{self, ServiceResponse},
    error::ErrorInternalServerError,
    middleware::ErrorHandlerResponse,
    BaseHttpResponse,
};
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

use super::api::error;
#[derive(Error, Debug)]
#[error("{0:#?}")]
pub struct GeneralError(Box<dyn std::error::Error + Send + Sync + 'static>);

generic_handler_err!(GeneralError, GeneralError);

impl ResponseError for GeneralError {
    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{:#?}", self.to_string());
        // BaseHttpResponse::build(self.status_code())
        //     .content_type(mime::TEXT_HTML_UTF_8)
        //     .body(template::ErrorSite::html())
        todo!()
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

pub fn render_500<B>(
    res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    // let new_res = ServiceResponse::new(
    //     res.request().clone(),
    //     HttpResponse::InternalServerError()
    //         .body(template::ErrorSite::html())
    //         .into_body(),
    // );

    // Ok(ErrorHandlerResponse::Response(new_res))
    let new_res = ServiceResponse::new(
        res.request().clone(),
        HttpResponse::InternalServerError().body("500").into_body(),
    );

    Ok(ErrorHandlerResponse::Response(new_res))
}
