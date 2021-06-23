use crate::{
    auth::{
        error::AuthError,
        login::{validate_login_form, LoginForm},
        register::{new_account, validate_register_form, RegisterForm},
    },
    State,
};
use crate::{
    hide_internal, inc_sql,
    model::{user::Account, Permission},
    session_storage,
};

use actix_web::{web, HttpResponse, ResponseError};

use actix_session::Session;
use actix_web::http::StatusCode;
use tokio_pg_mapper::FromTokioPostgresRow;

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AuthError::EmailOrUsernameExists => StatusCode::CONFLICT,
            AuthError::Validation(_) => StatusCode::BAD_REQUEST,
            AuthError::UserNotFound => StatusCode::NOT_FOUND,
            AuthError::Unauthorized | AuthError::WrongPassword | AuthError::BadForm(_) => {
                StatusCode::UNAUTHORIZED
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    
    fn error_response(&self) -> HttpResponse {
        log::error!("{}", self.to_string());
        crate::json_error!(AuthError, self)
    }
}

#[derive(Debug, serde::Serialize)]
struct AuthJsonError<'a> {
    message: &'a str,
    status: u16,
}

pub async fn register(
    state: web::Data<State>,
    form: Result<web::Json<RegisterForm>, actix_web::Error>,
) -> Result<HttpResponse, AuthError> {
    let form = form?.into_inner();
    validate_register_form(&form)?;
    let mut client = state.db_pool.get().await?;
    new_account(&mut client, &form, Permission::User).await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn logout(session: Session) -> HttpResponse {
    session_storage::forget(&session);
    //redirect("/login")
    HttpResponse::Ok().finish()
}

pub async fn login(
    state: web::Data<State>,
    session: Session,
    form: Result<web::Json<LoginForm>, actix_web::Error>,
) -> Result<HttpResponse, AuthError> {
    let form = form?.into_inner();
    validate_login_form(&form)?;
    let client = state.db_pool.get().await?;
    let stmt = client.prepare(inc_sql!("get/account")).await?;
    let row = client
        .query_one(&stmt, &[&form.email])
        .await
        .map_err(|_| AuthError::UserNotFound)?;
    let account: Account = Account::from_row(row)?;
    if bcrypt::verify(&form.password, &account.password_hash()).unwrap() {
        //id.remember(account.account_name.clone());
        account.save(&session).map_err(|_| AuthError::Session)?;
        Ok(HttpResponse::Ok().json(account))
    } else {
        Err(AuthError::WrongPassword)
    }
}

pub async fn user_info(
    _state: web::Data<State>,
    session: Session,
) -> Result<HttpResponse, AuthError> {
    let account = Account::from_session(&session).ok_or_else(|| AuthError::Unauthorized)?;

    Ok(HttpResponse::Ok().json(account))
}

// #[cfg(test)]
// mod tests {
//     // use super::*;
//     use actix_web::{http::HeaderValue, test, web, App};

//     use super::*;
//     #[tokio::test]
//     async fn test_login_get() {
//         let mut app =
//             test::init_service(App::default().route("/web/login", web::get().to(login_site))).await;
//         let req = test::TestRequest::get().uri("/web/login").to_request();
//         let resp = test::call_service(&mut app, req).await;
//         assert_eq!(
//             Some(&HeaderValue::from_static("text/html")),
//             resp.headers().get("content-type")
//         );
//     }
// }
