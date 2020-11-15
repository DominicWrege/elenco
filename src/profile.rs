use crate::auth::get_session;
use actix_session::Session;
use actix_web::HttpResponse;
use askama::Template;
#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileSite {
    username: String,
    status: bool,
}

pub async fn site(session: Session) -> HttpResponse {
    //let username = id.identity().unwrap_or(String::from("Username.."));
    let username = get_session(&session)
        .and_then(|s| Some(s.username))
        .unwrap_or(String::from("Default Username"));
    HttpResponse::Ok().content_type("text/html").body(
        ProfileSite {
            username,
            status: true,
        }
        .render()
        .unwrap(),
    )
}
