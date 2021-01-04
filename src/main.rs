use actix_web::{
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
            .configure(routes::api)
            .configure(routes::web)
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
        eprintln!("{:#?}", e);
        std::process::exit(1);
    }
}
