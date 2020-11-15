use actix_session::Session;
use actix_web::{dev::HttpResponseBuilder, http, web, HttpResponse, ResponseError};
use askama::Template;
use thiserror::Error;
use url::Url;

use crate::State;
#[derive(Template)]
#[template(path = "feed_form.html")]
struct Feed<'a> {
    feed_content: Option<FeedContent<'a>>,
    status: bool,
    error_msg: Option<String>,
}

impl<'a> Feed<'a> {
    pub fn new(
        title: &'a str,
        img: Option<Url>,
        description: &'a str,
        url: &'a Url,
        author: &'a str,
        err: Option<String>,
    ) -> Feed<'a> {
        Feed {
            feed_content: Some(FeedContent {
                img,
                title,
                description,
                url,
                author,
            }),
            status: true,
            error_msg: err,
        }
    }
}
#[derive(Debug)]
pub struct FeedContent<'a> {
    pub url: &'a Url,
    pub img: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: &'a str,
}

pub async fn feed_form() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        Feed {
            feed_content: None,
            status: true,
            error_msg: None,
        }
        .render()
        .unwrap(),
    )
}

#[derive(serde::Deserialize)]
pub struct FeedForm {
    #[serde(rename = "feed-url")]
    pub feed: Url,
}
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("An Inavalid RSS Feed was provided!. {0}")]
    InvalidRssFeed(#[from] rss::Error),
    #[error("{0}")]
    Connection(#[from] reqwest::Error),
    #[error("DB error: {0}")]
    DB(#[from] anyhow::Error),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .content_type("text/html")
            .body(
                Feed {
                    feed_content: None,
                    status: true,
                    error_msg: Some(self.to_string()),
                }
                .render()
                .unwrap(),
            )
    }
}
// TODO better err handling
pub async fn submit_feed(form: web::Form<FeedForm>) -> Result<HttpResponse, HttpError> {
    let resp_bytes = reqwest::get(form.feed.clone())
        .await?
        .bytes()
        .await
        .unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let podcast_feed = into_feed(&channel, &form.feed);

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(podcast_feed.render().unwrap()))
}

pub async fn handle_submit_feed(
    form: web::Form<FeedForm>,
    state: web::Data<State>,
    ses: Session,
) -> Result<HttpResponse, HttpError> {
    use super::auth::get_session;
    use super::db::save_feed;
    let id = get_session(&ses).unwrap().id;
    let resp_bytes = reqwest::get(form.feed.clone())
        .await?
        .bytes()
        .await
        .unwrap();
    let feed_bytes = std::io::Cursor::new(&resp_bytes);
    let channel = rss::Channel::read_from(feed_bytes)?;
    let podcast_feed = into_feed(&channel, &form.feed);
    save_feed(&state.db_pool, &podcast_feed.feed_content.unwrap(), id).await?;
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, "/api/feeds")
        .finish())
}

fn into_feed<'a>(feed: &'a rss::Channel, url: &'a Url) -> Feed<'a> {
    let img = feed
        .image()
        .and_then(|img| Url::parse(img.url()).ok())
        .or_else(|| {
            feed.itunes_ext()
                .and_then(|it| it.image().and_then(|u| Url::parse(u).ok()))
        });
    let author = feed
        .itunes_ext()
        .and_then(|x| x.author())
        .unwrap_or_default();
    Feed::new(feed.title(), img, feed.description(), url, author, None)
}
