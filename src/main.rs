use actix_session::CookieSession;
use actix_web::{cookie::SameSite, middleware, middleware::Logger, web, App, HttpServer};
//use sqlx::PgPool;
mod auth_middleware;
mod db;
mod handler;
mod routes;
mod util;
use deadpool_postgres::Pool;
use rand::Rng;
mod model;
mod podcast_util;
mod template;
#[derive(Clone)]
pub struct State {
    db_pool: Pool,
}
async fn run() -> Result<(), anyhow::Error> {
    let state = State {
        db_pool: db::util::connect_and_migrate().await?,
    };
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    let _private_key = rand::thread_rng().gen::<[u8; 32]>();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(|| util::redirect("/login")))
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/search/{title}")
                            .route(web::get().to(handler::api::feeds_by_name)),
                    )
                    .service(web::resource("/feeds").route(web::get().to(handler::api::all_feeds))),
            )
            .service(
                web::scope("/web")
                    .service(actix_files::Files::new("/static", "./static").show_files_listing())
                    .wrap(auth_middleware::CheckLogin)
                    .wrap(
                        CookieSession::private(&[1; 32])
                            .name("auth")
                            .secure(false)
                            .max_age_time(time::Duration::days(2))
                            .lazy(true)
                            .path("/web")
                            .same_site(SameSite::Strict)
                            .lazy(true),
                    )
                    .configure(routes::register_auth_routes),
            )
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
