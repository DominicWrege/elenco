use std::{collections::BTreeSet, convert::TryFrom};

use chrono::{DateTime, Duration, Utc};
use reqwest::Url;

#[derive(Debug)]
pub struct PreviewFeedContent<'a> {
    pub url: &'a Url,
    pub img: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: String,
    pub episodes: Vec<EpisodePreview<'a>>,
}
#[derive(Debug)]
pub struct RawFeed<'a> {
    pub url: &'a Url,
    pub img_path: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: Option<&'a str>,
    pub episodes: Vec<EpisodeRow>,
    pub subtitle: Option<&'a str>,
    pub language: Option<&'a str>,
    pub link_web: Url,
    pub categories: BTreeSet<&'a str>,
}

impl<'a> RawFeed<'a> {
    pub fn link_web(&self) -> &str {
        self.link_web.as_str()
    }
    pub fn url(&self) -> &str {
        self.link_web.as_str()
    }
    pub fn img_path(&self) -> Option<&str> {
        self.img_path.as_ref().and_then(|i| Some(i.as_str()))
    }
}
#[derive(Debug)]
pub struct EpisodeRow {
    pub title: String,
    pub description: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<String>>,
    pub duration: Option<u32>,
    pub show_notes: Option<String>,
    pub url: Option<Url>,
    pub media_url: Url,
}

#[derive(Debug)]
pub struct EpisodePreview<'a> {
    pub title: &'a str,
    pub link: Option<Url>,
    pub duration: &'a str,
}
// TODO use
impl<'a> RawFeed<'a> {
    pub fn try_from_channel(feed: &'a rss::Channel, url: &'a Url) -> Result<Self, anyhow::Error> {
        use super::podcast::{episode_list, parse_img_url};
        let mut categories_set = BTreeSet::new();
        for category in feed.categories() {
            if !category.name().is_empty() {
                categories_set.insert(category.name());
            }
        }
        // TODO think about itunes subcategories?!
        if let Some(categories) = feed.itunes_ext().map(|it| it.categories()) {
            for category in categories {
                if !category.text().is_empty() {
                    categories_set.insert(category.text());
                }
            }
        }
        Ok(Self {
            url,
            img_path: parse_img_url(&feed),
            title: feed.title(),
            description: feed.description(),
            author: feed.itunes_ext().and_then(|it| it.author()),
            episodes: episode_list(&feed),
            subtitle: feed.itunes_ext().and_then(|it| it.subtitle()),
            language: feed.language(), // better lang codes?!
            link_web: Url::parse(feed.link())?,
            categories: categories_set,
        })
    }
}

// TODO return Err if shity input
impl<'a> TryFrom<&'a rss::Item> for EpisodeRow {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            title: item.title().unwrap().to_string(),
            description: item.description().map(|d| d.into()),
            published: item.pub_date().and_then(|d| parse_datetime_rfc822(d).ok()),
            keywords: item.itunes_ext().and_then(|it| it.keywords()).map(|k| {
                String::from(k)
                    .split(",")
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
            }),
            duration: item
                .itunes_ext()
                .and_then(|it| it.duration())
                .and_then(|d| parse_duration_from_str(d))
                .map(|x| x.num_seconds() as u32),
            show_notes: item.content().map(|s| s.to_string()).or_else(|| {
                item.itunes_ext()
                    .and_then(|it| it.summary().and_then(|su| Some(su.into())))
            }),
            url: item.link().and_then(|u| Url::parse(u).ok()),
            media_url: item
                .enclosure()
                .and_then(|e| Url::parse(e.url()).ok())
                .unwrap(),
        })
    }
}
impl<'a> TryFrom<&'a rss::Item> for EpisodePreview<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            title: item.title().unwrap_or_default(),
            link: item.link().and_then(|u| Url::parse(u).ok()),
            duration: item
                .itunes_ext()
                .and_then(|o| o.duration())
                .unwrap_or_default(),
        })
    }
}

// after RFC https://tools.ietf.org/html/rfc822
fn parse_datetime_rfc822(stamp: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    DateTime::parse_from_rfc2822(stamp).map(|t| t.into())
}

fn digit_thing(s: &str) -> Option<i64> {
    match s.len() {
        2 => match s {
            "00" => Some(0),
            _ => s.trim_start_matches('0').parse::<u8>().ok().map(i64::from),
        },
        1 => s.parse().ok(),
        0 | _ => None,
    }
}

fn parse_duration_from_str(s: &str) -> Option<Duration> {
    let digits = s.split(':').collect::<Vec<_>>();

    let (h, m, s) = match digits.as_slice() {
        [h, m, s] if s.len() == 2 => (Some(h), m, s),
        [m, s] if s.len() == 2 => (None, m, s),
        _ => return None,
    };
    let hours = Duration::hours(digit_thing(h.unwrap_or(&"0"))?);
    let minutes = digit_thing(m)?;
    let seconds = digit_thing(s)?;
    if minutes >= 60 || seconds >= 60 {
        return None;
    };

    Some(hours + Duration::minutes(minutes) + Duration::seconds(seconds))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_utc_datetime() {
        let datetimes = [
            "Fri, 23 Oct 2020 03:00:00 -0000",
            "Tue, 17 Jul 2018 03:00:00 +0000",
            "Mon, 23 Nov 2020 22:07:46 GMT",
            "Wed, 18 Nov 2020 11:00:00 -0000",
            "Wed, 25 Nov 2020 17:00:00 +0000",
            "Tue, 24 Nov 2020 05:00:00 +0000",
            "Sun, 08 Nov 2020 13:48:44 GMT",
            "Wed, 20 Mar 2019 16:12:26 +0000",
        ];

        for time in &datetimes {
            parse_datetime_rfc822(time).expect(time);
        }
    }
    //The duration should be in one of the following formats: HH:MM:SS, H:MM:SS, MM:SS, M:SS.
    #[test]
    fn test_duration() {
        let ok = [
            ("00:00", Duration::seconds(0)),
            ("1:00:00", Duration::hours(1)),
            ("01:00:00", Duration::hours(1)),
            ("02:30:00", Duration::hours(2) + Duration::minutes(30)),
            (
                "12:45:05",
                Duration::hours(12) + Duration::minutes(45) + Duration::seconds(5),
            ),
            ("00:03", Duration::seconds(3)),
            ("27:19", Duration::minutes(27) + Duration::seconds(19)),
            ("0:03", Duration::seconds(3)),
            ("00:44:38", Duration::minutes(44) + Duration::seconds(38)),
            ("0:44:38", Duration::minutes(44) + Duration::seconds(38)),
        ];
        let fail = [
            "60:00", "90", "90:14", "00:420", "420:00", "90:210", "7:1", "0:0",
        ];

        for (time, exp) in &ok {
            assert_eq!(
                parse_duration_from_str(time),
                Some(*exp),
                "parsed from: {}",
                time
            );
        }

        for time in &fail {
            assert_eq!(parse_duration_from_str(time), None, "parsed from: {}", time);
        }
    }
}
