use crate::model::Feed;
use crate::model::{Permission, RawFeed};
use actix_web::{http::StatusCode, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};

#[derive(Template, Default)]
#[template(path = "register.html")]
pub struct Register<'a> {
    error_msg: Option<&'a str>,
    permission: Option<Permission>,
}

#[derive(Template, Default)]
#[template(path = "login.html")]
pub struct Login<'a> {
    error_msg: Option<&'a str>,
    permission: Option<Permission>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileSite {
    pub username: String,
    pub permission: Option<Permission>,
    pub submitted_feeds: Vec<Feed>,
}
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorSite {
    pub status_code: StatusCode,
    pub permission: Option<Permission>,
}

#[derive(Template)]
#[template(path = "register_moderator.html")]
pub struct RegisterModerator {
    pub permission: Option<Permission>,
}

pub trait LoginRegister: TemplateIntoResponse + Template {
    fn response(&self, status_code: StatusCode) -> Result<HttpResponse, askama::Error> {
        Ok(HttpResponse::build(status_code)
            .content_type("text/html")
            .body(self.render()?))
    }
}

impl<'a> LoginRegister for Login<'a> {}
impl<'a> LoginRegister for Register<'a> {}

impl<'a> Login<'a> {
    pub fn error_msg(&'a mut self, msg: &'a str) -> &Self {
        self.error_msg = Some(msg);
        self
    }
}

impl<'a> Register<'a> {
    pub fn error_msg(&'a mut self, msg: &'a str) -> &Self {
        self.error_msg = Some(msg);
        self
    }
}

#[derive(Template, Debug)]
#[template(path = "feed_form.html")]
pub struct FeedPreviewSite<'a> {
    pub metadata: Option<RawFeed<'a>>,
    pub permission: Option<Permission>,
    pub error_msg: Option<String>,
}

#[derive(Template, Debug)]
#[template(path = "moderator.html")]
pub struct ModeratorSite {
    pub permission: Option<Permission>,
    pub queued_feeds: Vec<Feed>,
    pub review_feeds: Vec<Feed>,
}
