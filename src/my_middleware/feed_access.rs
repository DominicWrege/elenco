use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use crate::{handler::error::log_error, inc_sql, model::Permission, util::page_not_found};
use actix_session::UserSession;
use actix_web::Error;
use actix_web::Result;
use actix_web::{dev::Transform, web};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse},
    HttpResponse,
};
use anyhow::anyhow;
use futures_util::future::{ok, Future, Ready};

pub struct FeedAccess;

impl<S> Transform<S, ServiceRequest> for FeedAccess
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = FeedAccessMidldleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(FeedAccessMidldleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct FeedAccessMidldleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S> Service<ServiceRequest> for FeedAccessMidldleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        use crate::model::user::Account;
        Box::pin(async move {
            let state = req
                .app_data::<web::Data<crate::State>>()
                .ok_or_else(|| log_error(anyhow!("state error")))?;

            let client = state.db_pool.get().await.map_err(log_error)?;
            let account = Account::from_session(&req.get_session())
                .ok_or_else(|| log_error(anyhow!("session error")))?;
            match account.permission() {
                Permission::Admin => srv.call(req).await,
                Permission::User => {
                    if let Ok(feed_id) = &req.match_info().path()[1..].parse::<i32>() {
                        let submitter_check_stmnt = client
                            .prepare(inc_sql!("get/feed/submitter_check"))
                            .await
                            .map_err(log_error)?;
                        if client
                            .query_one(&submitter_check_stmnt, &[&feed_id, &account.id()])
                            .await
                            .is_err()
                        {
                            log::warn!("Access to the feed was denied.");
                            let resp = page_not_found();
                            return Ok(req.into_response(resp));
                        }
                    } else {
                        let response = HttpResponse::BadRequest().finish();
                        return Ok(req.into_response(response));
                    }
                    srv.call(req).await
                }
            }
        })
    }
}
