use crate::util::LanguageCodeLookup;
use crate::{
    handler::feed_detail::EpisodeSmall, session_storage::SessionContext,
    time_date::DurationFormator,
};
use crate::{
    handler::{manage::ModeratorFeed, profile::ProfileFeed},
    model::{self, channel::Feed, Permission, Status},
};
use actix_web::{http::StatusCode, HttpResponse};
use askama_actix::{Template, TemplateToResponse};
use chrono::{DateTime, Utc};

#[derive(Template, Default)]
#[template(path = "auth/register.html")]
pub struct Register<'a> {
    error_msg: Option<&'a str>,
    session_context: Option<SessionContext>,
}

#[derive(Template, Default)]
#[template(path = "auth/login.html")]
pub struct Login<'a> {
    error_msg: Option<&'a str>,
    session_context: Option<SessionContext>,
}

#[derive(Template)]
#[template(path = "profile.html")]
pub struct ProfileSite {
    pub session_context: Option<SessionContext>,
    pub submitted_feeds: Vec<ProfileFeed>,
}
#[derive(Template)]
#[template(path = "error/general.html")]
pub struct ErrorSite {
    pub status_code: StatusCode,
    pub session_context: Option<SessionContext>,
}

impl ErrorSite {
    pub fn html() -> String {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            session_context: None,
        }
        .render()
        .unwrap()
    }
}

#[derive(Template)]
#[template(path = "auth/register_moderator.html")]
pub struct RegisterModerator {
    pub session_context: Option<SessionContext>,
}

pub trait LoginRegister: TemplateToResponse + Template {
    fn response(&self, status_code: StatusCode) -> Result<HttpResponse, askama::Error> {
        Ok(HttpResponse::build(status_code)
            .content_type(mime::TEXT_HTML_UTF_8)
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
    pub feed: Feed<'a>,
    pub feed_exists: bool,
}

#[derive(Template, Debug)]
#[template(path = "feed/feed_form.html")]
pub struct FeedPreviewSite<'a> {
    pub context: Option<Context<'a>>,
    pub session_context: Option<SessionContext>,
    pub error_msg: Option<String>,
}

#[derive(Template, Debug)]
#[template(path = "moderator.html")]
pub struct ModeratorSite {
    pub username: String,
    pub session_context: Option<SessionContext>,
    pub queued_feeds: Vec<ModeratorFeed>,
    pub review_feeds: Vec<ModeratorFeed>,
}

#[derive(Template, Debug)]
#[template(path = "feed/feed_table_row.html")]
pub struct ModeratorFeedTableRow {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub author_name: String,
    pub link_web: Option<String>,
    pub submitted: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub username: String,
}

#[derive(Template, Debug, Default)]
#[template(path = "error/not_found.html")]
pub struct NotFound {
    pub session_context: Option<SessionContext>,
}

impl NotFound {
    pub fn render_response(session: &actix_session::Session) -> HttpResponse {
        let html = Self {
            session_context: SessionContext::from(&session),
        }
        .render()
        .unwrap();
        HttpResponse::build(StatusCode::NOT_FOUND).body(html)
    }
}
#[derive(Template, Debug)]
#[template(path = "feed/feed_detail.html")]
pub struct FeedDetailSite {
    pub session_context: Option<SessionContext>,
    pub feed: model::json::Feed,
    pub episodes: Vec<EpisodeSmall>,
}
