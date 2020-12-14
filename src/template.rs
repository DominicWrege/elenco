use crate::{
    model::{FeedSmall2, PreviewFeedContent},
    podcast::{episode_list, parse_author, parse_img_url},
};
use actix_web::{dev::HttpResponseBuilder, HttpResponse};
use askama::Template;
use reqwest::Url;

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

#[derive(Template, Debug)]
#[template(path = "feed_form.html")]
pub struct FeedPreviewSite<'a> {
    metadata: Option<PreviewFeedContent<'a>>,
    status: bool,
    error_msg: Option<String>,
}

impl<'a> FeedPreviewSite<'a> {
    pub fn new(
        metadata: Option<PreviewFeedContent<'a>>,
        err: Option<String>,
    ) -> FeedPreviewSite<'a> {
        FeedPreviewSite {
            metadata,
            status: true,
            error_msg: err,
        }
    }
    pub fn preview(feed: &'a rss::Channel, url: &'a Url) -> Self {
        FeedPreviewSite::new(
            Some(PreviewFeedContent {
                url,
                img: parse_img_url(&feed),
                title: feed.title(),
                description: feed.description(),
                author: parse_author(&feed),
                episodes: episode_list(&feed),
            }),
            None,
        )
    }
}
