use reqwest::Url;

// use postgres_types::{FromSql, ToSql};
use super::item::Episode;
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize)]
pub struct Feed<'a> {
    pub url: Url,
    pub img: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: Option<&'a str>,
    pub episodes: Vec<Episode<'a>>,
    pub subtitle: Option<&'a str>,
    pub language_code: Option<&'a str>,
    pub link_web: Option<Url>,
    pub categories: BTreeMap<&'a str, Vec<&'a str>>,
}

impl<'a> Feed<'a> {
    pub fn link_web(&self) -> Option<&str> {
        self.link_web.as_ref().map(|link| link.as_str())
    }
    pub fn url(&self) -> &str {
        self.url.as_str()
    }
    pub fn parse(feed: &'a rss::Channel, url: Url) -> Self {
        let itunes_summary = feed.itunes_ext().and_then(|it| it.summary());
        let x = (itunes_summary, feed.description());
        let description = match x {
            (Some(itunes_summary), description)
                if !description.is_empty() && !itunes_summary.is_empty() =>
            {
                if itunes_summary.len() > description.len() {
                    itunes_summary
                } else {
                    description
                }
            }
            (None, description) if !description.is_empty() => description,
            _ => "default description",
        };

        Self {
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
            description,
            author: feed
                .itunes_ext()
                .and_then(|it| it.author())
                .or(Some("Default Author")),
            episodes: Episode::from_items(&feed.items()),
            subtitle: parse_subtitle(&feed),
            language_code: feed.language().map(|code| &code[..2]),
            categories: parse_categories(&feed),
        }
    }
}

fn parse_categories(feed: &'_ rss::Channel) -> BTreeMap<&str, Vec<&str>> {
    let mut categories_map = BTreeMap::new();

    for category in feed.categories() {
        if !category.name().is_empty() {
            categories_map.insert(category.name(), Vec::new());
        }
    }
    if let Some(categories) = feed.itunes_ext().map(|it| it.categories()) {
        for category in categories {
            if !category.text().trim().is_empty() {
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
    let web_link = Url::parse(feed.link()).ok()?;
    if (feed_url.scheme() == "http" || feed_url.scheme() == "https")
        && feed_url.host() == web_link.host()
        && feed_url.path() == web_link.path()
        && feed_url.query() == web_link.query()
    {
        None
    } else {
        Url::parse(feed.link()).ok()
    }
}

fn parse_subtitle(feed: &rss::Channel) -> Option<&str> {
    let parsed_subtitle = feed.itunes_ext().and_then(|it| it.subtitle());

    if Some(feed.description()) == parsed_subtitle {
        return None;
    } else if let Some(text) = parsed_subtitle {
        if text.trim().is_empty() {
            return None;
        }
    }

    parsed_subtitle
}
