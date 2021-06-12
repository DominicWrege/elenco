use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::{
    cookie::SameSite,
    http, middleware,
    middleware::{ErrorHandlers, Logger},
    web::{self},
    App, HttpServer,
};
use handler::general_error::render_500;
use img_cache::ImageCache;

mod auth;
mod db;
mod handler;
mod routes;
mod util;
use deadpool_postgres::Pool;
use rand::Rng;
mod img_cache;
mod macros;
mod model;
mod my_middleware;
mod path;
mod session_storage;
mod socket;
mod time_date;

pub type Client = deadpool_postgres::Client<tokio_postgres::NoTls>;

#[derive(Clone)]
pub struct State {
    db_pool: Pool<tokio_postgres::NoTls>,
    img_cache: ImageCache,
}

async fn run() -> Result<(), anyhow::Error> {
    let state = State {
        db_pool: db::util::connect_and_migrate().await?,
        img_cache: ImageCache::new("img-cache").await?,
    };
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let _private_key = rand::thread_rng().gen::<[u8; 32]>();
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(
                CookieSession::private(&[1; 32])
                    .name("auth")
                    .path("/")
                    .secure(false)
                    .http_only(false)
                    .max_age(chrono::Duration::days(2).num_seconds())
                    .same_site(SameSite::Lax)
                    .lazy(true),
            )
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .supports_credentials()
                    .allow_any_header() // fix me
                    .allowed_methods(vec!["GET", "POST"]),
            )
            .wrap(middleware::Compress::default())
            .wrap(
                Logger::new("ip: %a status: %s time: %Dms req: %r")
                    .exclude_regex("^(/static/|/web/img/)"),
            )
            .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500))
            .route(
                "/img/{file_name:.+(jpeg|jpg|png)$}",
                web::get().to(handler::serve_img),
            )
            .configure(routes::api)
            .configure(routes::auth)
            .configure(routes::user)
            .configure(routes::moderator)
        //   .default_service(web::route().to(handler::general_error::not_found))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;
    Ok(())
}
#[actix_web::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{:#?}", e);
        std::process::exit(1);
    }
}
