use actix_web::{dev::HttpResponseBuilder, web, HttpResponse, ResponseError};
use askama::Template;
use thiserror::Error;
use url::Url;
#[derive(Template)]
#[template(path = "feed_form.html")]
struct PodcastFeed<'a> {
    feed_content: Option<FeedContent<'a>>,
    status: bool,
    error_msg: Option<String>,
}

impl<'a> PodcastFeed<'a> {
    pub fn new(
        title: &'a str,
        img: Option<Url>,
        description: &'a str,
        url: &'a Url,
        author: &'a str,
        err: Option<String>,
    ) -> PodcastFeed<'a> {
        PodcastFeed {
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

struct FeedContent<'a> {
    url: &'a Url,
    img: Option<Url>,
    title: &'a str,
    description: &'a str,
    author: &'a str,
}

pub async fn feed_form() -> HttpResponse {
    HttpResponse::Ok().content_type("text/html").body(
        PodcastFeed {
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
    #[error("An Inavalid RSS Feed was provided!")]
    InvalidRssFeed(#[from] rss::Error),
}

impl ResponseError for HttpError {
    fn status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .content_type("text/html")
            .body(
                PodcastFeed {
                    feed_content: None,
                    status: true,
                    error_msg: Some(self.to_string()),
                }
                .render()
                .unwrap(),
            )
    }
}

pub async fn submit_feed(form: web::Form<FeedForm>) -> Result<HttpResponse, HttpError> {
    let resp_bytes = reqwest::get(form.feed.clone())
        .await
        .unwrap()
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

fn into_feed<'a>(feed: &'a rss::Channel, url: &'a Url) -> PodcastFeed<'a> {
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
    PodcastFeed::new(feed.title(), img, feed.description(), url, author, None)
}
