use actix_session::CookieSession;
use actix_web::{
    cookie::SameSite,
    middleware,
    middleware::Logger,
    web::{self},
    App, HttpServer,
};

use img_cache::ImageCache;
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
mod session_storage;
mod template;
mod time_date;
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
    env_logger::init();
    let _private_key = rand::thread_rng().gen::<[u8; 32]>();
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(Logger::new("ip: %a status: %s time: %Dms req: %r"))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(|| util::redirect("/login")))
            .service(web::scope("/api").configure(routes::api))
            .service(
                web::scope("/web")
                    .wrap(
                        CookieSession::private(&[1; 32])
                            .name("auth")
                            .secure(false)
                            .max_age(chrono::Duration::days(2).num_seconds())
                            .lazy(true)
                            .path("/web/auth")
                            .same_site(SameSite::Strict)
                            .lazy(true),
                    )
                    .route(
                        "/img/{filename:.+(jpeg|jpg|png)$}",
                        web::get().to(handler::serve_img),
                    )
                    .configure(routes::login_register)
                    .route("404", web::get().to(handler::general_error::not_found))
                    .service(
                        web::scope("/auth")
                            .wrap(my_middleware::auth::CheckLogin)
                            .route("/feed/{feed_id}", web::get().to(handler::feed_detail::site))
                            .configure(routes::user)
                            .configure(routes::admin),
                    ),
            )
            .default_service(web::route().to(handler::general_error::not_found))
    })
    .bind("0.0.0.0:8080")?
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
