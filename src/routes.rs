use crate::{
    handler::auth::{login, login_site, logout, register, register_site},
    handler::{self, podcast, profile},
    my_middleware,
};

use actix_web::web;
pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/logout").to(logout))
        .service(web::resource("/profile").route(web::get().to(profile::site)))
        .service(
            web::resource("/new-feed")
                .route(web::get().to(podcast::feed_form))
                .route(web::post().to(podcast::feed_preview)),
        )
        .service(web::resource("/save-feed").route(web::post().to(podcast::save_feed)));
}

pub fn login_register(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/register")
            .route(web::post().to(register))
            .route(web::get().to(register_site)),
    )
    .service(
        web::resource("/login")
            .route(web::get().to(login_site))
            .route(web::post().to(login)),
    );
}

pub fn admin(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .wrap(my_middleware::admin::Moderator)
            .route("/manage", web::get().to(handler::moderator::manage))
            .route(
                "/update-feed-status",
                web::patch().to(handler::moderator::review_feed),
            ),
    );
}
