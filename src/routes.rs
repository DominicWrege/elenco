use crate::{
    handler::auth::{login, login_site, logout, register, register_site},
    handler::{self, feed_preview, profile},
    my_middleware,
};

use actix_web::web;
pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/logout").to(logout))
        .service(web::resource("/profile").route(web::get().to(profile::site)))
        .service(
            web::resource("/new-feed")
                .route(web::get().to(feed_preview::feed_form))
                .route(web::post().to(feed_preview::feed_preview)),
        )
        .service(web::resource("/save-feed").route(web::post().to(feed_preview::save_feed)));
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
            .wrap(my_middleware::moderator::Moderator)
            .route("/manage", web::get().to(handler::moderator::manage))
            .route(
                "/update-feed-status",
                web::patch().to(handler::moderator::review_feed),
            )
            .service(
                web::resource("register")
                    .route(web::post().to(handler::moderator::register))
                    .route(web::get().to(handler::moderator::register_site)),
            ),
    );
}

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/search/{title}").route(web::get().to(handler::api::feeds_by_name)))
        .service(web::resource("/feeds").route(web::get().to(handler::api::all_feeds)))
        .route("feed/{id}", web::get().to(handler::api::feeds_by))
        .route("/categories", web::get().to(handler::api::all_categories))
        .route(
            "/category/{category_id_name}",
            web::get().to(handler::api::category_by),
        );
}
