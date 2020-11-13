use crate::{util::redirect, State};
use actix_web::{dev::HttpResponseBuilder, web, HttpResponse, ResponseError};
use askama::Template;
//use postgres_types::{FromSql, ToSql};
use actix_identity::Identity;
use actix_web::http::StatusCode;
use email_address::EmailAddress;
use serde::Deserialize;
use thiserror::Error;
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("row was not found: {0}")]
    RowNotFound(#[from] tokio_postgres::error::Error),
    #[error("username or email is already taken")]
    EmailOrUsernameExists,
    #[error("could not map type")]
    Mapper(#[from] tokio_pg_mapper::Error),
    #[error("internal error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
}
#[derive(Debug, Error)]
pub enum LoginError {
    #[error("Wrong password")]
    WrongPassword,
    #[error("User does not exist")]
    UserNotFound,
    #[error("Internal error: {0}")]
    Mapper(#[from] tokio_pg_mapper::Error),
    #[error("Internal error: {0}")]
    Sql(#[from] tokio_postgres::Error),
    #[error("Internal error: {0}")]
    PoolError(#[from] deadpool_postgres::PoolError),
}

impl ResponseError for RegisterError {
    fn status_code(&self) -> StatusCode {
        match self {
            RegisterError::EmailOrUsernameExists => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        render_template(
            TemplateName::Register,
            Some(&self.to_string()),
            self.status_code(),
        )
    }
}

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            LoginError::WrongPassword | LoginError::UserNotFound => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        render_template(
            TemplateName::Login,
            Some(&self.to_string()),
            self.status_code(),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    email: String,
    password_check: String,
}

#[derive(Template)]
#[template(path = "register_login.html")]
pub struct RegisterLogin<'a> {
    error_msg: Option<&'a str>,
    template: TemplateName,
    status: bool,
}

#[derive(std::cmp::PartialEq)]
pub enum TemplateName {
    Login,
    Register,
}

fn render_template(
    tn: TemplateName,
    error_msg: Option<&str>,
    status_code: actix_web::http::StatusCode,
) -> HttpResponse {
    let rl = RegisterLogin {
        error_msg,
        template: tn,
        status: false,
    }
    .render()
    .unwrap();

    HttpResponseBuilder::new(status_code)
        .content_type("text/html")
        .body(rl)
}

pub async fn register_site() -> HttpResponse {
    render_template(TemplateName::Register, None, StatusCode::OK)
}

fn bad_resquest(err_msg: &str) -> HttpResponse {
    render_template(
        TemplateName::Register,
        Some(err_msg),
        StatusCode::BAD_REQUEST,
    )
}

pub async fn register(
    state: web::Data<State>,
    form: web::Form<RegisterForm>,
) -> Result<HttpResponse, RegisterError> {
    if form.password != form.password_check {
        return Ok(bad_resquest("A password repeat does not match password"));
    } else if form.username.is_empty() {
        return Ok(bad_resquest("A username is required"));
    } else if !EmailAddress::is_valid(&form.email) {
        return Ok(bad_resquest("This email address is not valid"));
    } else if form.password.is_empty() || form.password_check.is_empty() {
        return Ok(bad_resquest("A password is required"));
    }
    let client = state.db_pool.get().await?;
    let pwd_hash = bcrypt::hash(&form.password, 8).unwrap();
    let stmt = client
        .prepare("INSERT INTO Account(account_name, password_hash, email) Values($1, $2, $3)")
        .await?;
    client
        .execute(&stmt, &[&form.username, &pwd_hash, &form.email])
        .await
        .map_err(|_e| RegisterError::EmailOrUsernameExists)?;
    Ok(redirect("/login"))
}

pub async fn login_site() -> HttpResponse {
    render_template(TemplateName::Login, None, StatusCode::OK)
}
pub async fn logout(id: Identity) -> HttpResponse {
    id.forget();
    redirect("/login")
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    password: String,
    email: String,
}

pub async fn login(
    state: web::Data<State>,
    id: Identity,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, LoginError> {
    if form.password.is_empty() {
        return Ok(bad_resquest("A username is required"));
    } else if form.email.is_empty() {
        return Ok(bad_resquest("A email address is required"));
    }
    let client = state.db_pool.get().await?;
    let stmt = client
        .prepare("SELECT account_name, email, password_hash FROM Account WHERE email = $1")
        .await?;
    let row = client
        .query_one(&stmt, &[&form.email])
        .await
        .map_err(|_| LoginError::UserNotFound)?;
    let account: Account = Account::from_row(row)?;
    if bcrypt::verify(&form.password, &account.password_hash).unwrap() {
        id.remember(account.account_name.clone());
        Ok(redirect("/profile"))
    } else {
        Err(LoginError::WrongPassword.into())
    }
}

#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "account")]
struct Account {
    pub account_name: String,
    pub email: String,
    pub password_hash: String,
}
