use crate::model::{
    feed::{Feed, RawFeed},
    Permission,
};
use actix_web::{http::StatusCode, HttpResponse};
use askama_actix::{Template, TemplateIntoResponse};

#[derive(Template, Default)]
#[template(path = "auth/register.html")]
pub struct Register<'a> {
    error_msg: Option<&'a str>,
    permission: Option<Permission>,
}

#[derive(Template, Default)]
#[template(path = "auth/login.html")]
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
#[template(path = "error/general.html")]
pub struct ErrorSite {
    pub status_code: StatusCode,
    pub permission: Option<Permission>,
}

#[derive(Template)]
#[template(path = "auth/register_moderator.html")]
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
#[derive(Debug)]
pub struct Context<'a> {
    pub feed: RawFeed<'a>,
    pub feed_exists: bool,
}

#[derive(Template, Debug)]
#[template(path = "feed/feed_form.html")]
pub struct FeedPreviewSite<'a> {
    pub context: Option<Context<'a>>,
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

#[derive(Template, Debug, Default)]
#[template(path = "error/not_found.html")]
pub struct NotFound {
    pub permission: Option<Permission>,
}
