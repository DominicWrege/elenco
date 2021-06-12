use std::fmt::Display;

use crate::State;
use crate::{
    generic_handler_err, hide_internal, inc_sql,
    model::{Account, Permission},
    session_storage,
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

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("username or email is already taken")]
    EmailOrUsernameExists,
    #[error("{0}")]
    Validation(#[from] ValidationError),
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
    #[error("{0}")]
    BadForm(#[from] actix_web::Error),
}

generic_handler_err!(AuthError, AuthError::Internal);

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

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        log::error!("{}", self.to_string());

        crate::json_error!(AuthError, self)
    }
}

#[derive(Debug, serde::Serialize)]
struct AuthJsonError<'a> {
    message: &'a str,
    status: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterForm {
    username: String,
    email: String,
    password: String,
    password_check: String,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("The Passwords are not the same")]
    PasswordMisMatch,
    #[error("Invalid email address")]
    InvalidEmail,
    #[error("The Password is to short. It should be a least 4 chars long")]
    PasswordShort(String),
}

fn validate_register_form(form: &RegisterForm) -> Result<(), ValidationError> {
    let pwd_len = 4;
    let RegisterForm {
        email,
        password,
        password_check,
        ..
    } = form;

    match (email, password, password_check) {
        (_, password, password_check) if password != password_check => {
            Err(ValidationError::PasswordMisMatch)
        }
        (email, _, _) if email_address::EmailAddress::is_valid(&email) == false => {
            Err(ValidationError::InvalidEmail)
        }
        (_, password, password_check)
            if password.len() < pwd_len || password_check.len() < pwd_len =>
        {
            Err(ValidationError::PasswordShort(password.clone()))
        }
        _ => Ok(()),
    }
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
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    password: String,
    email: String,
}

fn validate_login_form(form: &LoginForm) -> Result<(), ValidationError> {
    match (&form.password, &form.email) {
        (_, email) if email_address::EmailAddress::is_valid(&email) == false => {
            Err(ValidationError::InvalidEmail)
        }
        (password, _) if password.is_empty() => {
            Err(ValidationError::PasswordShort(password.clone()))
        }
        _ => Ok(()),
    }
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
