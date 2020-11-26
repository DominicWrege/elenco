use crate::{
    auth::{login, login_site, logout, register, register_site},
    podcast, profile,
};

use actix_web::web;

pub fn register_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
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
    .service(web::resource("/save-feed").route(web::post().to(podcast::handle_submit_feed)));
}
