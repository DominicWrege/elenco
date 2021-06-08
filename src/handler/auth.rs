use std::fmt::Display;

use crate::State;
use crate::{
    generic_handler_err, hide_internal, inc_sql,
    model::{Account, Permission},
    session_storage, validation_handler_err, wrap_err,
};
use actix_web::{body::Body, web, BaseHttpResponse, HttpResponse, ResponseError};
use tokio_postgres::Client;
//use postgres_types::{FromSql, ToSql};
//use actix_identity::Identity;
use actix_session::Session;
use actix_web::http::StatusCode;
use serde::Deserialize;
use thiserror::Error;
use tokio_pg_mapper::FromTokioPostgresRow;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("username or email is already taken")]
    EmailOrUsernameExists,
    #[error("{0}")]
    Validation(ValiderError),
    #[error("Wrong password")]
    WrongPassword,
    #[error("User does not exist")]
    UserNotFound,
    #[error("Internal error: {0:#?}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
    #[error("Could not create session")]
    Session,
    #[error("Unauthorized or the session has expired")]
    Unauthorized,
}

generic_handler_err!(AuthError, AuthError::Internal);
validation_handler_err!(AuthError, AuthError::Validation);

wrap_err!(ValidationErrors, ValiderError);
#[derive(Debug)]
pub struct ValiderError(ValidationErrors);

impl Display for ValiderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors = self.0.field_errors();
        let first_error_msg = errors
            .into_iter()
            .flat_map(|(_, b)| b)
            .take(1)
            .filter_map(|v| v.message.clone())
            .collect::<String>();
        write!(f, "{:}", first_error_msg)
    }
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match *self {
            AuthError::EmailOrUsernameExists => StatusCode::CONFLICT,
            AuthError::Validation(_) => StatusCode::BAD_REQUEST,
            AuthError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        let msg = hide_internal!(AuthError, self);

        let json = AuthJsonError {
            message: &msg,
            status: self.status_code().as_u16(),
        }
        .to_json_string();

        BaseHttpResponse::build(self.status_code())
            .content_type(mime::APPLICATION_JSON)
            .body(Body::from(json))
    }
}

#[derive(Debug, serde::Serialize)]
struct AuthJsonError<'a> {
    message: &'a str,
    status: u16,
}

impl AuthJsonError<'_> {
    fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterForm {
    #[validate(length(min = 1, message = "An username is required"))]
    username: String,
    #[validate(
        length(min = 1, message = "An username is required"),
        must_match = "password_check"
    )]
    password: String,
    #[validate(email(message = "An username is required"))]
    email: String,

    #[validate(must_match(other = "password", message = "Passwords are not the same"))]
    password_check: String,
}

pub async fn register(
    state: web::Data<State>,
    form: web::Json<RegisterForm>,
) -> Result<HttpResponse, AuthError> {
    form.validate()?;
    let mut client = state.db_pool.get().await?;
    new_account(&mut client, &form, Permission::User).await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn new_account(
    client: &mut Client,
    form: &RegisterForm,
    permission: Permission,
) -> Result<(), AuthError> {
    let trx = client.transaction().await?;
    let pwd_hash =
        bcrypt::hash(&form.password, 8).map_err(|err| AuthError::Internal(err.into()))?;

    let stmt = trx
        .prepare("INSERT INTO Account(username, password_hash, email, account_type) Values($1, $2, $3, $4)")
        .await?;
    trx.execute(
        &stmt,
        &[&form.username, &pwd_hash, &form.email, &permission],
    )
    .await
    .map_err(|_e| AuthError::EmailOrUsernameExists)?;
    trx.commit().await?;
    Ok(())
}

pub async fn logout(session: Session) -> HttpResponse {
    session_storage::forget(&session);
    //redirect("/login")
    HttpResponse::Ok().finish()
}
#[derive(Debug, Validate, Deserialize)]
pub struct LoginForm {
    #[validate(length(min = 1, message = "An username is required"))]
    password: String,
    #[validate(email(message = "An email address is required"))]
    email: String,
}

pub async fn login(
    state: web::Data<State>,
    session: Session,
    form: web::Json<LoginForm>,
) -> Result<HttpResponse, AuthError> {
    form.validate()?;
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
