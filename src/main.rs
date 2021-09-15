use actix_cors::Cors;
use actix_session::CookieSession;
use actix_web::{
    cookie::SameSite,
    http, middleware,
    middleware::Logger,
    web::{self, Data},
    App, HttpServer,
};
// use handler::general_error::render_500;
use img_cache::ImageCache;

mod auth;
mod db;
mod handler;
mod routes;
mod util;
use deadpool_postgres::Pool;
mod img_cache;
mod macros;
mod model;
mod my_middleware;
mod path;
mod socket;
mod time_date;

pub type Client = deadpool_postgres::Client;

#[derive(Clone)]
pub struct State {
    db_pool: Pool,
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

    let cookie_config = CookieConfig::new();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(state.clone()))
            .wrap(
                CookieSession::private(&cookie_config.key)
                    .name("auth")
                    .domain(&cookie_config.domain)
                    .path("/")
                    .secure(cookie_config.secure)
                    .http_only(false)
                    .max_age(chrono::Duration::days(2).num_seconds())
                    .same_site(cookie_config.same_site)
                    .lazy(true),
            )
            .wrap(middleware::Compress::default())
            .wrap(
                Cors::default()
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                        http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        http::header::CONTENT_TYPE,
                    ])
                    .allow_any_origin()
                    .supports_credentials()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"]),
            )
            .wrap(
                Logger::new("ip: %a status: %s time: %Dms req: %r")
                    .exclude_regex("^(/static/|/web/img/)"),
            )
            .route(
                "/img/{file_name:.+(jpeg|jpg|png)$}",
                web::get().to(handler::serve_img),
            )
            .configure(routes::api)
            .configure(routes::auth)
            .configure(routes::user)
            .configure(routes::moderator)
            .default_service(web::route().to(handler::error::not_found))
    })
    .bind("0.0.0.0:8020")?
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

#[derive(Clone)]
pub struct CookieConfig {
    secure: bool,
    key: [u8; 32],
    domain: String,
    same_site: SameSite,
}

impl CookieConfig {
    pub fn new() -> Self {
        let domain = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".into());
        log::info!("using cookie domain {}", domain);
        if cfg!(debug_assertions) {
            Self {
                secure: false,
                key: [0u8; 32],
                domain,
                same_site: SameSite::Lax,
            }
        } else {
            Self {
                secure: true,
                domain,
                key: rand::random(),
                same_site: SameSite::Lax,
            }
        }
    }
}
