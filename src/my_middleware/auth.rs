//! Middleware that checks if the user is logged in and if not redirects it to the login page.
use std::task::{Context, Poll};

use crate::util;
use actix_session::UserSession;
use actix_web::dev::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures_util::future::{ok, Either, Ready};
pub struct CheckLogin;

impl<S, B> Transform<S, ServiceRequest> for CheckLogin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = CheckLoginMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckLoginMiddleware { service })
    }
}
pub struct CheckLoginMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for CheckLoginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        use crate::model::Account;
        match Account::from_session(&req.get_session()) {
            Some(_) => Either::Left(self.service.call(req)),
            None => Either::Right(ok(req.into_response(util::redirect("/login").into_body()))),
        }
    }
}
