use crate::template::{self, TemplateName};
use crate::{util::redirect, State};
use actix_web::{web, HttpResponse, ResponseError};
//use postgres_types::{FromSql, ToSql};
//use actix_identity::Identity;
use actix_session::Session;
use actix_web::http::StatusCode;
use email_address::EmailAddress;
use serde::{Deserialize, Serialize};
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
        template::RegisterLogin::new(TemplateName::Register, Some(&self.to_string()))
            .render_response(self.status_code())
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
        template::RegisterLogin::new(TemplateName::Login, Some(&self.to_string()))
            .render_response(self.status_code())
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    email: String,
    password_check: String,
}

pub async fn register_site() -> HttpResponse {
    template::RegisterLogin::new(TemplateName::Register, None).render_response(StatusCode::OK)
}

fn bad_request(err_msg: &str, template_name: TemplateName) -> HttpResponse {
    template::RegisterLogin::new(template_name, Some(err_msg))
        .render_response(StatusCode::BAD_REQUEST)
}

pub async fn register(
    state: web::Data<State>,
    form: web::Form<RegisterForm>,
) -> Result<HttpResponse, RegisterError> {
    if form.password != form.password_check {
        return Ok(bad_request(
            "A password repeat does not match password",
            TemplateName::Register,
        ));
    } else if form.username.is_empty() {
        return Ok(bad_request(
            "A username is required",
            TemplateName::Register,
        ));
    } else if !EmailAddress::is_valid(&form.email) {
        return Ok(bad_request(
            "This email address is not valid",
            TemplateName::Register,
        ));
    } else if form.password.is_empty() || form.password_check.is_empty() {
        return Ok(bad_request(
            "A password is required",
            TemplateName::Register,
        ));
    }
    let client = state.db_pool.get().await?;
    let pwd_hash = bcrypt::hash(&form.password, 8).unwrap();
    let stmt = client
        .prepare("INSERT INTO Account(username, password_hash, email) Values($1, $2, $3)")
        .await?;
    client
        .execute(&stmt, &[&form.username, &pwd_hash, &form.email])
        .await
        .map_err(|_e| RegisterError::EmailOrUsernameExists)?;
    Ok(redirect("/login"))
}

#[derive(Serialize, Debug, Deserialize)]
pub struct SessionStorage {
    pub username: String,
    pub id: i32,
}

pub async fn login_site() -> HttpResponse {
    template::RegisterLogin::new(TemplateName::Login, None).render_response(StatusCode::OK)
}
pub async fn logout(session: Session) -> HttpResponse {
    session.remove(SESSION_KEY_ACCOUNT);
    session.clear();
    redirect("/login")
}
const SESSION_KEY_ACCOUNT: &str = "account";
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    password: String,
    email: String,
}

pub fn get_session(s: &Session) -> Option<SessionStorage> {
    s.get::<SessionStorage>(SESSION_KEY_ACCOUNT).ok().flatten()
}

pub async fn login(
    state: web::Data<State>,
    session: Session,
    form: web::Form<LoginForm>,
) -> Result<HttpResponse, LoginError> {
    if form.password.is_empty() {
        return Ok(bad_request("A username is required", TemplateName::Login));
    } else if form.email.is_empty() {
        return Ok(bad_request(
            "A email address is required",
            TemplateName::Login,
        ));
    }
    let client = state.db_pool.get().await?;
    let stmt = client
        .prepare("SELECT username, email, id, password_hash FROM Account WHERE email = $1")
        .await?;
    let row = client
        .query_one(&stmt, &[&form.email])
        .await
        .map_err(|_| LoginError::UserNotFound)?;
    let account: Account = Account::from_row(row)?;
    if bcrypt::verify(&form.password, &account.password_hash).unwrap() {
        //id.remember(account.account_name.clone());
        session
            .set(
                SESSION_KEY_ACCOUNT,
                SessionStorage {
                    username: account.username,
                    id: account.id,
                },
            )
            .unwrap();
        Ok(redirect("/profile"))
    } else {
        Err(LoginError::WrongPassword.into())
    }
}

#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "account")]
struct Account {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub id: i32,
}
#[cfg(test)]
mod tests {
    // use super::*;
    use actix_web::{http::HeaderValue, test, web, App};

    use super::*;
    #[tokio::test]
    async fn test_login_get() {
        let mut app =
            test::init_service(App::new().route("/web/login", web::get().to(login_site))).await;
        let req = test::TestRequest::get().uri("/web/login").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert_eq!(
            Some(&HeaderValue::from_static("text/html")),
            resp.headers().get("content-type")
        );
    }
}
