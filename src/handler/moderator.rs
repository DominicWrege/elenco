use actix_web::web::Data;

use crate::{template, State};

pub async fn manage(_ses: actix_session::Session, _state: Data<State>) -> template::ModeratorSite {
    template::ModeratorSite { status: true }
}
