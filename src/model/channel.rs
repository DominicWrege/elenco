use reqwest::Url;

// use postgres_types::{FromSql, ToSql};
use super::item::EpisodeRow;
use std::collections::BTreeMap;

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
    pub link_web: Option<Url>,
    pub categories: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> RawFeed<'a> {
    pub fn link_web(&self) -> Option<&str> {
        self.link_web.as_ref().map(|link| link.as_str())
    }
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn parse(feed: &'a rss::Channel, url: Url) -> Result<Self, anyhow::Error> {
        Ok(Self {
            link_web: parse_website_link(&feed, &url),
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
            subtitle: parse_subtitle(&feed),
            language_code: feed.language().map(|code| &code[..2]),
            categories: parse_categories(&feed),
        })
    }
}

fn parse_categories<'a>(feed: &'a rss::Channel) -> BTreeMap<&str, Vec<&str>> {
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

fn parse_website_link(feed: &rss::Channel, feed_url: &Url) -> Option<Url> {
    if feed_url.as_str() == feed.link() {
        None
    } else {
        Url::parse(feed.link()).ok()
    }
}

fn parse_subtitle(feed: &rss::Channel) -> Option<&str> {
    let parsed_subtitle = feed.itunes_ext().and_then(|it| it.subtitle());
    if Some(feed.description()) == parsed_subtitle {
        None
    } else {
        parsed_subtitle
    }
}
