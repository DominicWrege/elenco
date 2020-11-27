use std::convert::{TryFrom, TryInto};

use actix_web::{dev::HttpResponseBuilder, HttpResponse, ResponseError};
use askama::Template;

use rss::Image;
use thiserror::Error;
use url::Url;

use crate::model::{EpisodePreview, PreviewFeedContent};
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
        author: String,
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

pub fn parse_img_url(feed: &rss::Channel) -> Option<Url> {
    feed.image()
        .and_then(|img| Url::parse(img.url()).ok())
        .or_else(|| {
            feed.itunes_ext()
                .and_then(|it| it.image().and_then(|u| Url::parse(u).ok()))
        })
}
fn parse_author(feed: &rss::Channel) -> String {
    feed.itunes_ext()
        .and_then(|x| x.author())
        .unwrap_or_default()
        .into()
}

pub fn episode_list<'a, T>(feed: &'a rss::Channel) -> Vec<T>
where
    T: TryFrom<&'a rss::Item>,
{
    feed.items().iter().flat_map(|item| item.try_into().ok()).collect()
}

fn generate_feed_preview<'a>(feed: &'a rss::Channel, url: &'a Url) -> NewFeedSite<'a> {
    let img = parse_img_url(&feed);

    // let episodes: Vec<_> = feed.items().iter().map(|item| item.into()).collect();
    let episodes = episode_list(&feed);
    NewFeedSite::new(
        feed.title(),
        img,
        feed.description(),
        url,
        parse_author(&feed),
        episodes,
        None,
    )
}

pub mod handler {
    use actix_web::{http, web, HttpResponse};

    use super::{generate_feed_preview, HttpError, NewFeedSite, Url};
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
        let podcast_feed = super::generate_feed_preview(&channel, &form.feed);
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
        let podcast_feed = generate_feed_preview(&channel, &form.feed);

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
