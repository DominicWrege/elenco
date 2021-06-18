use crate::{
    handler::{self, user_feed},
    my_middleware,
};
// TODO seperater /sub path!!
use actix_web::web::{self};
use handler::{api, auth, save_preview_feed};

pub fn user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .wrap(my_middleware::auth::CheckLogin)
            .route("/info", web::get().to(auth::user_info))
            .route("/feeds", web::get().to(user_feed::site))
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
            .route("/manage", web::get().to(handler::manage::manage))
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
    cfg.service(
        web::scope("/api")
            .default_service(web::route().to(api::error::not_found))
            .route(
                "/completion/{query}",
                web::route().to(api::feed::completion),
            )
            .service(
                web::scope("/feeds")
                    .route("", web::get().to(api::feed::all))
                    .route("/search", web::get().to(api::feed::search)),
            )
            .service(
                web::scope("/feed")
                    .route("/{id}", web::get().to(api::feed::by_name))
                    .route("/{id}/episodes", web::get().to(api::episode::by_feed_id))
                    .service(
                        web::scope("/") // change / to action
                            .wrap(my_middleware::auth::CheckLogin)
                            .route(
                                "preview",
                                web::post().to(save_preview_feed::preview::create),
                            )
                            .route("new", web::post().to(save_preview_feed::save::save))
                            .route("update/{id}", web::patch().to(user_feed::update_feed))
                            .route(
                                "subscribe",
                                web::post().to(handler::subscription::subscribe),
                            )
                            .route(
                                "unsubscribe",
                                web::post().to(handler::subscription::unsubscribe),
                            ),
                    ),
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

// pub fn register_routes(cfg: &mut web::ServiceConfig) {
//     cfg.service(
//         web::scope("/")

//             // .route("404", web::get().to(handler::general_error::not_found))
//       ,
//         // .service(
//         //     web::scope("/auth2")
//         //         // .service(
//         //         //     web::scope("/feed")
//         //         //         // .wrap(my_middleware::feed_access::FeedAccess)
//         //         //         .route("/{feed_id}", web::get().to(handler::feed_detail::site)),
//         //         // )

//         // ),
//     );
// }
