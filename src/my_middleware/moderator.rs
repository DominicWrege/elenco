use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_session::UserSession;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::Error;
use actix_web::{dev::Transform, web};
use futures_util::future::{ok, Future, Ready};

pub struct Moderator;

impl<S, B> Transform<S> for Moderator
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
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

impl<S, B> Service for ModeratorMiddleware<S>
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
        // let inner = self.inner.clone();
        use crate::db::is_moderator;
        use crate::model::Account;
        Box::pin(async move {
            let state = req.app_data::<web::Data<crate::State>>().unwrap();
            let db = &state.db_pool;
            let user_id = Account::get_account(&req.get_session()).unwrap().id();
            //TODO
            dbg!(&req.path());
            if let Ok(client) = db.get().await {
                if let Ok(true) = is_moderator(&client, user_id).await {
                    return srv.call(req).await;
                }
            }
            let resp = actix_web::HttpResponse::Forbidden().finish().into_body();
            Ok(req.into_response(resp))
        })
    }
}
