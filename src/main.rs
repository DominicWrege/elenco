use actix_session::CookieSession;
use actix_web::{cookie::SameSite, middleware, middleware::Logger, web, App, HttpServer};
//use sqlx::PgPool;
mod api;
mod auth;
mod auth_middleware;
mod db;
mod podcast;
mod profile;
mod routes;
mod util;
use deadpool_postgres::{Manager, Pool};
use rand::Rng;
mod model;
#[derive(Clone)]
pub struct State {
    db_pool: Pool,
}
async fn run() -> Result<(), anyhow::Error> {
    let state = State {
        db_pool: db::connect_and_migrate().await?,
    };
    std::env::set_var("RUST_LOG", "actix_web=info");
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
                        web::resource("/search/{title}").route(web::get().to(api::feeds_by_name)),
                    )
                    .service(web::resource("/feeds").route(web::get().to(api::all_feeds))),
            )
            .service(
                web::scope("/web")
                    .service(actix_files::Files::new("/static", "./static").show_files_listing())
                    .wrap(auth_middleware::CheckLogin)
                    .wrap(
                        CookieSession::private(&[1; 32])
                            .name("auth")
                            .secure(false)
                            .max_age_time(time::Duration::days(3))
                            .same_site(SameSite::Strict),
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
