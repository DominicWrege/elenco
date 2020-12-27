use chrono::offset::Utc;
use chrono::DateTime;
use reqwest::Url;
use std::convert::{TryFrom, TryInto};

use crate::time_date::{parse_datetime_rfc822, parse_duration_from_str};

use super::parse_explicit;
#[derive(Debug)]
pub struct EpisodeRow<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<&'a str>>,
    pub duration: Option<i64>,
    pub show_notes: Option<&'a str>,
    pub url: Option<Url>,
    pub media_url: Url,
    pub explicit: bool,
    pub guid: Option<&'a str>,
}
impl<'a> EpisodeRow<'a> {
    pub fn url(&self) -> Option<&str> {
        self.url.as_ref().and_then(|url| Some(url.as_str()))
    }
    pub fn media_url(&self) -> &str {
        self.media_url.as_str()
    }
    pub fn format_duration(&self) -> String {
        if let Some(seconds) = self.duration {
            let duration = chrono::Duration::seconds(seconds as i64);
            let seconds = duration.num_seconds() % 60;
            let minutes = (duration.num_seconds() / 60) % 60;
            let hours = duration.num_hours();

            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            "None".to_string()
        }
    }
    pub fn from(items: &[rss::Item]) -> Vec<EpisodeRow> {
        items.iter().flat_map(|item| item.try_into().ok()).collect()
    }
}

// TODO add Field Media Typ
// TODO check iuntes show notes
impl<'a> TryFrom<&'a rss::Item> for EpisodeRow<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            title: item
                .title()
                .ok_or_else(|| anyhow::format_err!("field title is required"))?,
            description: item.description().map(|d| d.into()),
            published: item.pub_date().and_then(|d| parse_datetime_rfc822(d).ok()),
            keywords: item
                .itunes_ext()
                .and_then(|itunes| itunes.keywords())
                .map(|k| k.split(",").collect::<Vec<_>>()),
            duration: item
                .itunes_ext()
                .and_then(|itunes| itunes.duration())
                .and_then(|d| parse_duration_from_str(d))
                .map(|x| x.whole_seconds() as i64),
            show_notes: item
                .content()
                .or_else(|| item.itunes_ext().and_then(|itunes| itunes.summary())),
            url: item.link().and_then(|u| Url::parse(u).ok()),
            media_url: item
                .enclosure()
                .and_then(|e| Url::parse(e.url()).ok())
                .ok_or_else(|| anyhow::format_err!("field enclosure is required"))?,
            explicit: parse_explicit(item.itunes_ext()),
            guid: item.guid().map(|g| g.value()),
        })
    }
}
