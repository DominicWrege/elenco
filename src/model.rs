use chrono::offset::Utc;
use chrono::{DateTime, Duration};
use reqwest::Url;
use std::{collections::BTreeMap, convert::TryFrom};
use tokio_pg_mapper_derive::PostgresMapper;
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
    pub url: Url,
    pub img_path: Option<Url>,
    pub title: &'a str,
    pub description: &'a str,
    pub author: Option<&'a str>,
    pub episodes: Vec<EpisodeRow<'a>>,
    pub subtitle: Option<&'a str>,
    pub language: Option<&'a str>,
    pub link_web: Url,
    pub categories: BTreeMap<&'a str, Vec<&'a str>>,
}

#[derive(Debug, PostgresMapper)]
#[pg_mapper(table = "feed")]
pub struct FeedSmall2 {
    pub title: String,
    pub img_path: String,
    pub author_name: String,
    pub link_web: String,
    pub status: String,
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
pub struct EpisodeRow<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<&'a str>>,
    pub duration: Option<i32>,
    pub show_notes: Option<&'a str>,
    pub url: Option<Url>,
    pub media_url: Url,
    pub explicit: bool,
    pub guid: Option<&'a str>,
}
impl<'a> EpisodeRow<'a> {
    pub fn url(&self) -> Option<&str> {
        self.url.as_ref().and_then(|i| Some(i.as_str()))
    }
    pub fn media_url(&self) -> &str {
        self.media_url.as_str()
    }
}
#[derive(Debug)]
pub struct EpisodePreview<'a> {
    pub title: &'a str,
    pub link: Option<Url>,
    pub duration: &'a str,
}
// TODO use

impl<'a> RawFeed<'a> {
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
}

impl<'a> TryFrom<&'a rss::Channel> for RawFeed<'a> {
    type Error = anyhow::Error;

    fn try_from(feed: &'a rss::Channel) -> Result<Self, Self::Error> {
        use super::podcast_util::{episode_list, parse_img_url};

        Ok(Self {
            url: Url::parse(feed.link())?,
            img_path: parse_img_url(&feed),
            title: feed.title(),
            description: feed.description(),
            author: feed.itunes_ext().and_then(|it| it.author()),
            episodes: episode_list(&feed),
            subtitle: feed.itunes_ext().and_then(|it| it.subtitle()),
            language: feed.language(), // better lang codes?!
            link_web: Url::parse(feed.link())?,
            categories: Self::parse_categories(&feed),
        })
    }
}

// TODO return Err if shity input
// TODO add Field Media Typ
// TODO check iuntes show notes
impl<'a> TryFrom<&'a rss::Item> for EpisodeRow<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            title: item.title().expect("No title, FIX ME!!!!!"),
            description: item.description().map(|d| d.into()),
            published: item.pub_date().and_then(|d| parse_datetime_rfc822(d).ok()),
            keywords: item
                .itunes_ext()
                .and_then(|it| it.keywords())
                .map(|k| k.split(",").collect::<Vec<_>>()),
            duration: item
                .itunes_ext()
                .and_then(|it| it.duration())
                .and_then(|d| parse_duration_from_str(d))
                .map(|x| x.num_seconds() as i32),
            show_notes: item.content().or_else(|| {
                item.itunes_ext()
                    .and_then(|it| it.summary().and_then(|su| Some(su.into())))
            }),
            url: item.link().and_then(|u| Url::parse(u).ok()),
            media_url: item
                .enclosure()
                .and_then(|e| Url::parse(e.url()).ok())
                .expect("No Media, FIX ME!!!!!"),
            explicit: parse_explicit(item.itunes_ext()),
            guid: item.guid().map(|g| g.value()),
        })
    }
}

fn parse_explicit(it_ext: Option<&rss::extension::itunes::ITunesItemExtension>) -> bool {
    matches!(
        it_ext.and_then(|ext| ext.explicit()),
        Some("Yes") | Some("yes") | Some("true") | Some("True") | Some("explicit")
    )
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
        2 | 3 => match s {
            "00" => Some(0),
            _ => s.trim_start_matches('0').parse::<u16>().ok().map(i64::from),
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
    let mut hours = digit_thing(h.unwrap_or(&"0"))?;
    let mut minutes = digit_thing(m)?;
    if 60 <= minutes {
        hours = minutes / 60;
        minutes = minutes % 60;
    };
    let seconds = digit_thing(s)?;
    if seconds >= 60 {
        return None;
    };
    Some(Duration::hours(hours) + Duration::minutes(minutes) + Duration::seconds(seconds))
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
    //The duration should be in one of the following formats: HH:MM:SS, H:MM:SS, MM:SS, M:SS and MMM::SS
    #[test]
    fn test_duration() {
        let ok = [
            ("00:00", Duration::seconds(0)),
            ("1:00:00", Duration::hours(1)),
            ("01:00:00", Duration::hours(1)),
            (
                "143:45",
                Duration::hours(2) + Duration::minutes(23) + Duration::seconds(45),
            ),
            (
                "218:11",
                Duration::hours(3) + Duration::minutes(38) + Duration::seconds(11),
            ),
            ("60:00", Duration::hours(1)),
            ("02:30:00", Duration::hours(2) + Duration::minutes(30)),
            ("360:00", Duration::hours(6)),
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
        let fail = ["90", "90:999", "00:420", "000:420", "90:210", "7:1", "0:0"];

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
