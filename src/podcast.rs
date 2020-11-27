use actix_web::{dev::HttpResponseBuilder, HttpResponse, ResponseError};
use askama::Template;
use chrono::{DateTime, Utc};
use thiserror::Error;
use url::Url;
#[derive(Template)]
#[template(path = "feed_form.html")]
struct NewFeedSite<'a> {
    metadata: Option<PreviewFeedContent<'a>>,
    status: bool,
    error_msg: Option<String>,
}

impl<'a> NewFeedSite<'a> {
    pub fn new(
        title: &'a str,
        img: Option<Url>,
        description: &'a str,
        url: &'a Url,
        author: &'a str,
        episodes: Vec<EpisodePreview<'a>>,
        err: Option<String>,
    ) -> NewFeedSite<'a> {
        NewFeedSite {
            metadata: Some(PreviewFeedContent {
                url,
                img,
                title,
                description,
                author,
                episodes,
            }),
            status: true,
            error_msg: err,
        }
    }
}
#[derive(Debug)]
pub struct PreviewFeedContent<'a> {
    pub url: &'a Url,
    pub img: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: &'a str,
    pub episodes: Vec<EpisodePreview<'a>>,
}
#[derive(Debug)]
pub struct RawFeed<'a> {
    pub url: &'a Url,
    pub img_path: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: &'a str,
    pub episodes: Vec<EpisodePreview<'a>>,
    pub subtitle: Option<String>,
    pub language: Option<&'a str>,
    pub copyright: Option<&'a str>,
    pub categories: Option<Vec<&'a str>>,
}

#[derive(Debug)]
pub struct RawEpisode {
    pub title: String,
    pub description: String,
    pub published: DateTime<Utc>,
    pub keywords: Vec<String>,
    pub duration: u32,
    pub show_notes: u32,
    pub url: Option<String>,
    pub media_url: Url,
}

#[derive(Debug)]
pub struct EpisodePreview<'a> {
    pub title: &'a str,
    pub link: Option<Url>,
    pub duration: &'a str,
}

fn parse_epsiodes<'a>(item: &'a rss::Item) -> EpisodePreview<'a> {
    EpisodePreview {
        title: item.title().unwrap_or_default(),
        link: item.link().and_then(|u| Url::parse(u).ok()),
        duration: item
            .itunes_ext()
            .and_then(|o| o.duration())
            .unwrap_or_default(),
    }
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
                NewFeedSite {
                    metadata: None,
                    status: true,
                    error_msg: Some(self.to_string()),
                }
                .render()
                .unwrap(),
            )
    }
}
// TODO better err handling

fn parse_meta_feed<'a>(feed: &'a rss::Channel, url: &'a Url) -> NewFeedSite<'a> {
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

    let episodes: Vec<_> = feed
        .items()
        .iter()
        .map(|item| parse_epsiodes(item))
        .collect();

    NewFeedSite::new(
        feed.title(),
        img,
        feed.description(),
        url,
        author,
        episodes,
        None,
    )
}

pub mod handler {
    use actix_web::{http, web, HttpResponse};

    use super::{parse_meta_feed, HttpError, NewFeedSite, Url};
    use crate::State;
    use askama::Template;
    #[derive(serde::Deserialize)]
    pub struct FeedForm {
        #[serde(rename = "feed-url")]
        pub feed: Url,
    }

    pub async fn save_feed(
        form: web::Form<FeedForm>,
        state: web::Data<State>,
        ses: actix_session::Session,
    ) -> Result<HttpResponse, HttpError> {
        use crate::auth::get_session;
        use crate::db::save_feed;
        let id = get_session(&ses).unwrap().id;
        let resp_bytes = reqwest::get(form.feed.clone())
            .await?
            .bytes()
            .await
            .unwrap();
        let feed_bytes = std::io::Cursor::new(&resp_bytes);
        let channel = rss::Channel::read_from(feed_bytes)?;
        let podcast_feed = super::parse_meta_feed(&channel, &form.feed);
        save_feed(&state.db_pool, &podcast_feed.metadata.unwrap(), id).await?;
        Ok(HttpResponse::Found()
            .header(http::header::LOCATION, "/api/feeds")
            .finish())
    }

    pub async fn feed_preview(form: web::Form<FeedForm>) -> Result<HttpResponse, HttpError> {
        let resp_bytes = reqwest::get(form.feed.clone())
            .await?
            .bytes()
            .await
            .unwrap();
        let feed_bytes = std::io::Cursor::new(&resp_bytes);
        let channel = rss::Channel::read_from(feed_bytes)?;
        let podcast_feed = parse_meta_feed(&channel, &form.feed);

        Ok(HttpResponse::Ok()
            .content_type("text/html")
            .body(podcast_feed.render().unwrap()))
    }

    pub async fn feed_form() -> HttpResponse {
        HttpResponse::Ok().content_type("text/html").body(
            NewFeedSite {
                metadata: None,
                status: true,
                error_msg: None,
            }
            .render()
            .unwrap(),
        )
    }
}
