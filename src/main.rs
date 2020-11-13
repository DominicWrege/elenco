use actix_web::{cookie::SameSite, web, App, HttpServer};
//use sqlx::PgPool;
mod auth;
mod auth_middleware;
mod podcast;
mod profile;
mod util;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use auth::{login, login_site, logout, register, register_site};
use deadpool_postgres::{Manager, Pool};
use rand::Rng;
#[derive(Clone)]
pub struct State {
    db_pool: Pool,
}

async fn run() -> Result<(), anyhow::Error> {
    let state = State {
        db_pool: connect_db().await?,
    };
    let _private_key = rand::thread_rng().gen::<[u8; 32]>();
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .wrap(auth_middleware::CheckLogin)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[1; 32])
                    .name("auth")
                    .secure(false)
                    .max_age_time(time::Duration::days(3))
                    .same_site(SameSite::Strict),
            ))
            .service(web::resource("/").to(login))
            .service(
                web::resource("/register")
                    .route(web::post().to(register))
                    .route(web::get().to(register_site)),
            )
            .service(
                web::resource("/login")
                    .route(web::post().to(login))
                    .route(web::get().to(login_site)),
            )
            .service(web::resource("/logout").to(logout))
            .service(web::resource("/profile").route(web::get().to(profile::site)))
            .service(
                web::resource("/new-feed")
                    .route(web::get().to(podcast::feed_form))
                    .route(web::post().to(podcast::submit_feed)),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;
    Ok(())
}
#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn connect_db() -> Result<Pool, anyhow::Error> {
    let mut pg_config = tokio_postgres::Config::default();
    pg_config
        .user("harra")
        .password("hund")
        .dbname("podcast")
        .host("127.0.0.1");
    let mngr = Manager::new(pg_config.clone(), tokio_postgres::NoTls);
    Ok(Pool::new(mngr, 12))
}
