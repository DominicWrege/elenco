use crate::{auth::get_session, db::get_feeds_for_account, template::ProfileSite, State};
use actix_session::Session;
use actix_web::{web, HttpResponse};
use askama::Template;

// TODO replace unwrap
pub async fn site(session: Session, state: web::Data<State>) -> HttpResponse {
    //let username = id.identity().unwrap_or(String::from("Username.."));

    let session_storage = get_session(&session).unwrap();
    let (id, username) = (session_storage.id, session_storage.username);

    HttpResponse::Ok().content_type("text/html").body(
        ProfileSite {
            username,
            status: true,
            submitted_feeds: get_feeds_for_account(&state.db_pool.get().await.unwrap(), id)
                .await
                .unwrap(),
        }
        .render()
        .unwrap(),
    )
}
