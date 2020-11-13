//! Middleware that checks if the user is logged in and if not redirects it to the login page.
use std::task::{Context, Poll};

use crate::util;
use actix_identity::RequestIdentity;
use actix_web::dev::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use futures_util::future::{ok, Either, Ready};
pub struct CheckLogin;

impl<S, B> Transform<S> for CheckLogin
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
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

//const NO_LOGIN_PATH: [&str; 3] = ["/login", "/static", "/register"];

impl<S, B> Service for CheckLoginMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        match req.get_identity() {
            Some(_) => Either::Left(self.service.call(req)),
            None if req.path().contains("/login")
                || req.path().starts_with("/static")
                || req.path().starts_with("/register") =>
            {
                Either::Left(self.service.call(req))
            }
            None => Either::Right(ok(req.into_response(util::redirect("/login").into_body()))),
        }
    }
}
