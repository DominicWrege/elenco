use crate::{
    handler::{self, user},
    my_middleware,
};
use actix_web::web::{self};
use handler::{auth, save_preview_feed};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(my_middleware::auth::CheckLogin)
            .route("/info", web::get().to(auth::user_info))
            .route("/feeds", web::get().to(user::submitted_feeds)),
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
            .service(
                web::scope("/review")
                    .route(
                        "/unassigned",
                        web::get().to(handler::manage::all_unassigned),
                    )
                    .route("/inbox", web::get().to(handler::manage::reviewer_inbox))
                    .route(
                        "/assign",
                        web::patch().to(handler::manage::assign_for_review),
                    )
                    .route("/reviewed", web::get().to(handler::manage::reviewed))
                    .route("", web::patch().to(handler::manage::review_feed)),
            )
            .service(web::scope("/socket").route(
                "/unassigned",
                web::get().to(handler::manage::register_socket),
            ))
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
    .route("/meta", web::get().to(handler::meta))
    .service(
        web::scope("/feeds")
            .route("/top", web::get().to(handler::feed::charts))
            .route("/recent", web::get().to(handler::feed::recent))
            .route("/search", web::get().to(handler::feed::search)),
    )
    .service(
        web::scope("/feed")
            .route("/{id}", web::get().to(handler::feed::by_name_or_id))
            .route("/{id}/related", web::get().to(handler::feed::related))
            .service(
                web::scope("/") // change / to action
                    .wrap(my_middleware::auth::CheckLogin)
                    .route(
                        "preview",
                        web::post().to(save_preview_feed::preview::create),
                    )
                    .route("new", web::post().to(save_preview_feed::save::save))
                    .route("update", web::patch().to(user::update_feed)),
            ),
    )
    .route(
        "/episode/{id}",
        web::get().to(handler::episode::by_episode_id),
    )
    .route(
        "/episodes/{feed_id}",
        web::get().to(handler::episode::by_feed_id),
    )
    .route(
        "/img-path/{feed_title}",
        web::get().to(handler::image_for_feed),
    )
    .service(
        web::scope("/subscription")
            .wrap(my_middleware::auth::CheckLogin)
            .service(
                web::resource("")
                    .route(web::post().to(handler::subscription::subscribe))
                    .route(web::delete().to(handler::subscription::unsubscribe)),
            )
            .service(
                web::resource("/user")
                    .route(web::get().to(user::subscriptions))
                    .route(web::post().to(handler::subscription::subscription_info)),
            ),
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
        web::resource("/comment")
            .wrap(my_middleware::auth::CheckLogin)
            .route(web::post().to(handler::comment::new)),
    )
    .route(
        "/comments/{id}",
        web::get().to(handler::comment::get_for_feed),
    );
}
