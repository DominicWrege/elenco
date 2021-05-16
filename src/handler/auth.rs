use std::fmt::Display;

use crate::{
    generic_handler_err, hide_internal, inc_sql,
    model::{Account, Permission},
    session_storage,
    template::{self},
    validation_handler_err, wrap_err,
};
use crate::{util::redirect, State};
use actix_web::{web, BaseHttpResponse, HttpResponse, ResponseError};
use tokio_postgres::Client;
//use postgres_types::{FromSql, ToSql};
//use actix_identity::Identity;
use crate::template::LoginRegister;
use actix_session::Session;
use actix_web::http::StatusCode;
use askama::Template;
use serde::Deserialize;
use template::Login;
use thiserror::Error;
use tokio_pg_mapper::FromTokioPostgresRow;
use validator::{Validate, ValidationErrors};

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("username or email is already taken")]
    EmailOrUsernameExists,
    #[error("{0}")]
    Validation(ValiderError),
    #[error("Internal error: {0}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Wrong password")]
    WrongPassword,
    #[error("User does not exist")]
    UserNotFound,
    #[error("Internal error: {0:#?}")]
    Internal(Box<dyn std::error::Error + Sync + Send>),
    #[error("{0}")]
    Validation(ValiderError),
    #[error("Could not create session")]
    Session,
}
generic_handler_err!(RegisterError, RegisterError::Internal);
generic_handler_err!(LoginError, LoginError::Internal);
validation_handler_err!(RegisterError, RegisterError::Validation);
validation_handler_err!(LoginError, LoginError::Validation);

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

impl ResponseError for RegisterError {
    fn status_code(&self) -> StatusCode {
        match *self {
            RegisterError::EmailOrUsernameExists => StatusCode::CONFLICT,
            RegisterError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        let msg = hide_internal!(RegisterError, self);

        let template = template::Register::default()
            .error_msg(&msg)
            .render()
            .unwrap();

        BaseHttpResponse::build(self.status_code())
            .content_type(mime::TEXT_HTML_UTF_8)
            .body(template)
    }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::WrongPassword | LoginError::UserNotFound | LoginError::Validation(_) => {
                StatusCode::UNAUTHORIZED
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    // fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
    //     log::error!("{}", self.to_string());
    //     let status = self.status_code();

    //     let body = serde_json::to_string(&JsonError {
    //         error: hide_internal!(ApiError, self),
    //         status: status.as_u16(),
    //     })
    //     .unwrap();

    //     actix_web::BaseHttpResponse::build(status)
    //         .content_type(mime::APPLICATION_JSON)
    //         .body(Body::from(body))
    // }

    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        let msg = hide_internal!(LoginError, self);
        BaseHttpResponse::build(self.status_code())
            .content_type(mime::TEXT_HTML_UTF_8)
            .body(Login::default().error_msg(&msg).render().unwrap())
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

pub async fn register_site() -> HttpResponse {
    template::Register::default()
        .response(StatusCode::OK)
        .unwrap()
}

pub async fn register(
    state: web::Data<State>,
    form: web::Form<RegisterForm>,
) -> Result<HttpResponse, RegisterError> {
    form.validate()?;
    let mut client = state.db_pool.get().await?;
    new_account(&mut client, &form, Permission::User).await?;
    Ok(redirect("/login"))
}

pub async fn new_account(
    client: &mut Client,
    form: &RegisterForm,
    permission: Permission,
) -> Result<(), RegisterError> {
    let trx = client.transaction().await?;
    let pwd_hash =
        bcrypt::hash(&form.password, 8).map_err(|err| RegisterError::Internal(err.into()))?;

    let stmt = trx
        .prepare("INSERT INTO Account(username, password_hash, email, account_type) Values($1, $2, $3, $4)")
        .await?;
    trx.execute(
        &stmt,
        &[&form.username, &pwd_hash, &form.email, &permission],
    )
    .await
    .map_err(|_e| RegisterError::EmailOrUsernameExists)?;
    trx.commit().await?;
    Ok(())
}

pub async fn login_site() -> HttpResponse {
    template::Login::default().response(StatusCode::OK).unwrap()
}
pub async fn logout(session: Session) -> HttpResponse {
    session_storage::forget(&session);
    redirect("/login")
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
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, LoginError> {
    form.validate()?;
    let client = state.db_pool.get().await?;
    let stmt = client.prepare(inc_sql!("get/account")).await?;
    let row = client
        .query_one(&stmt, &[&form.email])
        .await
        .map_err(|_| LoginError::UserNotFound)?;
    let account: Account = Account::from_row(row)?;
    if bcrypt::verify(&form.password, &account.password_hash()).unwrap() {
        //id.remember(account.account_name.clone());
        account.save(&session).map_err(|_| LoginError::Session)?;
        Ok(redirect("/auth/profile"))
    } else {
        Err(LoginError::WrongPassword)
    }
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
