use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_session::UserSession;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web::Result;
use actix_web::{dev::Transform, web};
use anyhow::anyhow;
use futures_util::future::{ok, Future, Ready};

use crate::handler::general_error::log_error;
pub struct Moderator;

impl<S, B> Transform<S, ServiceRequest> for Moderator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ModeratorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ModeratorMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct ModeratorMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service<ServiceRequest> for ModeratorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut srv = self.service.clone();
        use crate::db::is_moderator;
        use crate::model::Account;
        Box::pin(async move {
            let state = req
                .app_data::<web::Data<crate::State>>()
                .ok_or_else(|| log_error(anyhow!("State error")))?;
            let db = &state.db_pool;
            let user_id = Account::from_session(&req.get_session())
                .ok_or_else(|| log_error(anyhow!("Session error")))?
                .id();
            let client = db.get().await.map_err(log_error)?;

            if is_moderator(&client, user_id).await? {
                srv.call(req).await
            } else {
                log::warn!("User has no permission to access the moderator site.");
                let resp = actix_web::HttpResponse::Forbidden().finish().into_body();
                Ok(req.into_response(resp))
            }
        })
    }
}
