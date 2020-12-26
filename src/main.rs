use actix_session::CookieSession;
use actix_web::{cookie::SameSite, middleware, middleware::Logger, web, App, HttpServer};
//use sqlx::PgPool;
mod db;
mod handler;
mod routes;
mod util;
use deadpool_postgres::Pool;
use rand::Rng;
mod macros;
mod model;
mod my_middleware;
mod podcast_util;
mod session;
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
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(Logger::new("ip: %a status: %s time: %Dms req: %r"))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
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
                    .wrap(
                        CookieSession::private(&[1; 32])
                            .name("auth")
                            .secure(false)
                            .max_age_time(time::Duration::days(2))
                            .lazy(true)
                            .path("/web/auth")
                            .same_site(SameSite::Strict)
                            .lazy(true),
                    )
                    .configure(routes::login_register)
                    .service(
                        web::scope("/auth")
                            .wrap(my_middleware::auth::CheckLogin)
                            .configure(routes::user)
                            .configure(routes::admin),
                    ),
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
