use chrono::offset::Utc;
use chrono::DateTime;
use isolang::Language;
// use postgres_types::{FromSql, ToSql};
use reqwest::Url;
use serde::Serialize;
use std::collections::BTreeMap;
use tokio_pg_mapper_derive::PostgresMapper;

use super::episode::EpisodeRow;

#[derive(Debug, PostgresMapper, Serialize)]
#[pg_mapper(table = "feed")]
pub struct Feed {
    pub id: i32,
    pub url: String,
    pub title: String,
    #[serde(rename(serialize = "img"))]
    pub img_cache: Option<String>,
    // this is the creater not the submitter
    pub author_name: String,
    pub link_web: String,
    // TODO enum is better than string
    #[serde(skip_serializing)]
    pub status: String,
    #[serde(skip_serializing)]
    pub submitted: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub last_modified: DateTime<Utc>,
    pub description: String,
    pub language: String,
    #[serde(skip_serializing)]
    pub username: String,
}

#[derive(Debug)]
pub struct RawFeed<'a> {
    pub url: Url,
    pub img: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: Option<&'a str>,
    pub episodes: Vec<EpisodeRow<'a>>,
    pub subtitle: Option<&'a str>,
    pub language_code: Option<&'a str>,
    pub link_web: Url,
    pub categories: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> RawFeed<'a> {
    pub fn link_web(&self) -> &str {
        self.link_web.as_str()
    }
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    fn parse_categories(feed: &'a rss::Channel) -> BTreeMap<&str, Vec<&str>> {
        let mut categories_map = BTreeMap::new();

        for category in feed.categories() {
            if !category.name().is_empty() {
                categories_map.insert(category.name(), Vec::new());
            }
        }
        if let Some(categories) = feed.itunes_ext().map(|it| it.categories()) {
            for category in categories {
                if !category.text().is_empty() {
                    let sub_categories = categories_map
                        .entry(category.text())
                        .or_insert_with(Vec::new);
                    if let Some(sub_category) =
                        category.subcategory().filter(|sub| !sub.text().is_empty())
                    {
                        sub_categories.push(sub_category.text());
                    }
                }
            }
        }
        categories_map
    }

    pub fn parse(feed: &'a rss::Channel, url: Url) -> Result<Self, anyhow::Error> {
        Ok(Self {
            url,
            img: feed
                .image()
                .and_then(|img| Url::parse(img.url()).ok())
                .or_else(|| {
                    feed.itunes_ext()
                        .and_then(|itunes| itunes.image().and_then(|u| Url::parse(u).ok()))
                }),
            title: feed.title(),
            description: feed.description(),
            author: feed.itunes_ext().and_then(|it| it.author()),
            episodes: EpisodeRow::from(&feed.items()),
            subtitle: feed.itunes_ext().and_then(|it| it.subtitle()),
            language_code: feed.language().map(|code| &code[..2]),
            link_web: Url::parse(feed.link())?,
            categories: Self::parse_categories(&feed),
        })
    }
    pub fn language(&self) -> Option<Language> {
        self.language_code
            .and_then(|code| Language::from_639_1(code).or_else(|| Language::from_locale(code)))
    }
}
