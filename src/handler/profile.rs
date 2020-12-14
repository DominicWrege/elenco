use super::auth::get_session;
use crate::{db::api::get_feeds_for_account, template::ProfileSite, State};
use actix_session::Session;
use actix_web::web;
// TODO replace unwrap
pub async fn site(session: Session, state: web::Data<State>) -> ProfileSite {
    //let username = id.identity().unwrap_or(String::from("Username.."));

    let session_storage = get_session(&session).unwrap();
    let (id, username) = (session_storage.id, session_storage.username);

    ProfileSite {
        username,
        status: true,
        submitted_feeds: get_feeds_for_account(&state.db_pool.get().await.unwrap(), id)
            .await
            .unwrap(),
    }
}
