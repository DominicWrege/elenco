use crate::model::{FeedSmall2, PreviewFeedContent};
use actix_web::{dev::HttpResponseBuilder, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "register_login.html")]
pub struct RegisterLogin<'a> {
    error_msg: Option<&'a str>,
    template: TemplateName,
    status: bool,
}

impl<'a> RegisterLogin<'a> {
    pub fn new(template: TemplateName, error_msg: Option<&'a str>) -> Self {
        Self {
            template,
            error_msg,
            status: false,
        }
    }

    pub fn render_response(&self, status_code: actix_web::http::StatusCode) -> HttpResponse {
        HttpResponseBuilder::new(status_code)
            .content_type("text/html")
            .body(self.render().unwrap())
    }
}

#[derive(std::cmp::PartialEq)]
pub enum TemplateName {
    Login,
    Register,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileSite {
    pub username: String,
    pub status: bool,
    pub submitted_feeds: Vec<FeedSmall2>,
}

#[derive(Template)]
#[template(path = "feed_form.html")]
pub struct FeedPreviewSite<'a> {
    metadata: Option<PreviewFeedContent<'a>>,
    status: bool,
    error_msg: Option<String>,
}

impl<'a> FeedPreviewSite<'a> {
    pub fn new(metadata: Option<PreviewFeedContent<'a>>, err: Option<String>) -> FeedPreviewSite<'a> {
        FeedPreviewSite {
            metadata,
            status: true,
            error_msg: err,
        }
    }
}
