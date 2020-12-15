use crate::{db::api::get_feeds_for_account, template::ProfileSite, State};
use crate::{model::Permission, session::SessionStorage};
use actix_session::Session;
use actix_web::web;
// TODO replace unwrap
pub async fn site(session: Session, state: web::Data<State>) -> ProfileSite {
    //let username = id.identity().unwrap_or(String::from("Username.."));
    let (id, username) = SessionStorage::user(&session);
    ProfileSite {
        username,
        permission: Some(Permission::User),
        submitted_feeds: get_feeds_for_account(&state.db_pool.get().await.unwrap(), id)
            .await
            .unwrap(),
    }
}
