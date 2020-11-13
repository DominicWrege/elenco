use actix_identity::Identity;
use actix_web::HttpResponse;
use askama::Template;
#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileSite {
    username: String,
    status: bool,
}

pub async fn site(id: Identity) -> HttpResponse {
    let username = id.identity().unwrap_or(String::from("Username.."));
    dbg!(id.identity());
    HttpResponse::Ok().content_type("text/html").body(
        ProfileSite {
            username,
            status: true,
        }
        .render()
        .unwrap(),
    )
}
