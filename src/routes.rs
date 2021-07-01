use crate::{
    handler::{self, user},
    my_middleware,
};
// TODO seperater /sub path!!
use actix_web::web::{self};
use handler::{auth, save_preview_feed};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(my_middleware::auth::CheckLogin)
            .route("/info", web::get().to(auth::user_info))
            .route("/feeds", web::get().to(user::submitted_feeds))
            .route("/subscriptions", web::get().to(user::subscriptions))
            .route(
                "/has-subscription",
                web::post().to(handler::subscription::user_has_subscription),
            ),
    );
}

pub fn auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
            .service(
                web::scope("/")
                    .route("logout", web::post().to(auth::logout))
                    .wrap(my_middleware::auth::CheckLogin),
            ),
    );
}

pub fn moderator(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .wrap(my_middleware::auth::CheckLogin)
            .wrap(my_middleware::moderator::Moderator)
            .route("/inbox", web::get().to(handler::manage::inbox))
            .route(
                "/update-feed-status",
                web::patch().to(handler::manage::review_feed),
            )
            .route(
                "/fedd-live-update",
                web::get().to(handler::manage::register_socket),
            )
            .route(
                "/register",
                web::post().to(handler::manage::register_moderator),
            ),
    );
}

pub fn api(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/completion/{query}",
        web::route().to(handler::feed::completion),
    )
    .service(
        web::scope("/feeds")
            .route("", web::get().to(handler::feed::all))
            .route("/search", web::get().to(handler::feed::search)),
    )
    .service(
        web::scope("/feed")
            .route("/{id}", web::get().to(handler::feed::by_name_or_id))
            .route("/{id}/related", web::get().to(handler::feed::related))
            .route(
                "/{id}/episodes",
                web::get().to(handler::episode::by_feed_id),
            )
            .service(
                web::scope("/") // change / to action
                    .wrap(my_middleware::auth::CheckLogin)
                    .route(
                        "preview",
                        web::post().to(save_preview_feed::preview::create),
                    )
                    .route("new", web::post().to(save_preview_feed::save::save))
                    .route("update/{id}", web::patch().to(user::update_feed)),
            ),
    )
    .service(
        web::resource("/subscription")
            .wrap(my_middleware::auth::CheckLogin)
            .route(web::post().to(handler::subscription::subscribe))
            .route(web::delete().to(handler::subscription::unsubscribe)),
    )
    .route("/categories", web::get().to(handler::category::all))
    .service(
        web::scope("/category")
            .route(
                "/{category}",
                web::get().to(handler::category::by_id_or_name),
            )
            .route(
                "/{category}/feeds",
                web::get().to(handler::feed::by_category),
            ),
    )
    .route("/authors", web::get().to(handler::author::all))
    .route(
        "/author/{author_id_name}/feeds",
        web::get().to(handler::author::feeds),
    )
    .service(
        web::scope("/comment")
            .wrap(my_middleware::auth::CheckLogin)
            .route("", web::post().to(handler::comment::new))
            .route("/{id}", web::get().to(handler::comment::get_for_feed)),
    );
}
