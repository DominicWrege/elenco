use crate::{
    handler::auth::{login, login_site, logout, register, register_site},
    handler::{self, feed_preview, general_error::render_500, profile},
    my_middleware,
};

use actix_session::CookieSession;
use actix_web::{cookie::SameSite, http, middleware::ErrorHandlers, web};
use handler::{api, auth};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/logout").route(web::post().to(auth::logout)))
        .service(
            web::scope("/profile")
                .route("", web::get().to(profile::site))
                .route("update-feed", web::patch().to(profile::update_feed)),
        )
        .service(
            web::resource("/new-feed")
                .route(web::get().to(feed_preview::form_template))
                .route(web::post().to(feed_preview::create_preview)),
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
            .route("/manage", web::get().to(handler::manage::manage))
            .route(
                "/update-feed-status",
                web::patch().to(handler::manage::review_feed),
            )
            .route(
                "fedd-live-update",
                web::get().to(handler::manage::register_socket),
            )
            .service(
                web::resource("register")
                    .route(web::post().to(handler::manage::register_moderator))
                    .route(web::get().to(handler::manage::register_moderator_site)),
            ),
    );
}

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::scope("/user").route("/info", web::get().to(auth::user_info)))
            .default_service(web::route().to(api::error::not_found))
            .service(
                web::scope("/feeds")
                    .route("", web::get().to(api::feed::all))
                    .route("/search", web::get().to(api::feed::search)),
            )
            .service(
                web::scope("/feed")
                    .route("/{id}", web::get().to(api::feed::by_id))
                    .route("/{id}/episodes", web::get().to(api::episode::by_feed_id)),
            )
            .route("/categories", web::get().to(api::category::all))
            .service(
                web::scope("/category")
                    .route("/{category}", web::get().to(api::category::by_id_or_name))
                    .route("/{category}/feeds", web::get().to(api::feed::by_category)),
            )
            .route("/authors", web::get().to(api::author::all))
            .route(
                "/author/{author_id_name}/feeds",
                web::get().to(api::author::feeds),
            ),
    );
}

pub fn web(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/web")
            .wrap(ErrorHandlers::new().handler(http::StatusCode::INTERNAL_SERVER_ERROR, render_500))
            .route(
                "/img/{file_name:.+(jpeg|jpg|png)$}",
                web::get().to(handler::serve_img),
            )
            .configure(self::login_register)
            .route("404", web::get().to(handler::general_error::not_found))
            .service(
                web::scope("/auth")
                    .wrap(my_middleware::auth::CheckLogin)
                    .service(
                        web::scope("/feed")
                            .wrap(my_middleware::feed_access::FeedAccess)
                            .route("/{feed_id}", web::get().to(handler::feed_detail::site)),
                    )
                    .configure(self::user)
                    .configure(self::admin),
            ),
    );
}
