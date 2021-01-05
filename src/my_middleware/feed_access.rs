use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_session::UserSession;
use actix_web::Error;
use actix_web::{dev::Transform, web};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse},
    HttpResponse,
};
use futures_util::future::{ok, Future, Ready};

use crate::{inc_sql, model::Permission, util::page_not_found};

pub struct FeedAccess;

impl<S, B> Transform<S> for FeedAccess
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
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

impl<S, B> Service for FeedAccessMidldleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.borrow_mut().poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut srv = self.service.clone();
        use crate::model::Account;
        Box::pin(async move {
            let state = req.app_data::<web::Data<crate::State>>().unwrap();
            if let Ok(client) = state.db_pool.get().await {
                let account = Account::from_session(&req.get_session()).unwrap();
                // TODO FIX ME
                if let Ok(feed_id) = &req.match_info().path()[1..].parse::<i32>() {
                    if account.permission() != Permission::Admin {
                        if let Ok(submitter_check_stmnt) =
                            client.prepare(inc_sql!("get/feed/submitter_check")).await
                        {
                            if let Err(_) = client
                                .query_one(&submitter_check_stmnt, &[&feed_id, &account.id()])
                                .await
                            {
                                let resp = page_not_found().into_body();
                                return Ok(req.into_response(resp));
                            }
                        }
                    }
                } else {
                    let response = HttpResponse::BadRequest().finish().into_body();
                    return Ok(req.into_response(response));
                }
            }
            srv.call(req).await
        })
    }
}