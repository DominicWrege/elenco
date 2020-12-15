use actix_web::web::Data;

use crate::{model::Permission, template, State};

pub async fn manage(_ses: actix_session::Session, _state: Data<State>) -> template::ModeratorSite {
    template::ModeratorSite {
        permission: Some(Permission::Admin),
    }
}
