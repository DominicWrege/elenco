use crate::time_date::{parse_datetime_rfc822, parse_duration_from_str, DurationFormator};
use chrono::offset::Utc;
use chrono::DateTime;
use reqwest::Url;
use std::convert::{TryFrom, TryInto};

#[derive(Debug)]
pub struct Episode<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<&'a str>>,
    pub duration: Option<i64>,
    pub show_notes: Option<&'a str>,
    pub url: Option<Url>,
    pub media_url: Url,
    pub explicit: bool,
}
impl<'a> Episode<'a> {
    pub fn url(&self) -> Option<&str> {
        self.url.as_ref().map(|url| url.as_str())
    }
    pub fn media_url(&self) -> &str {
        self.media_url.as_str()
    }
    pub fn from_items(items: &[rss::Item]) -> Vec<Episode> {
        let mut items: Vec<Episode> = items.iter().flat_map(|item| item.try_into().ok()).collect();
        items.sort_by(|a, b| b.published.cmp(&a.published));
        items
    }
}

impl DurationFormator for Episode<'_> {
    fn duration(&self) -> Option<i64> {
        self.duration
    }
}

// TODO add Field Media Typ
// TODO check iuntes show notes
impl<'a> TryFrom<&'a rss::Item> for Episode<'a> {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            title: item
                .title()
                .ok_or_else(|| anyhow::format_err!("field title is required"))?,
            description: item.description(),
            published: item.pub_date().and_then(|d| parse_datetime_rfc822(d).ok()),
            keywords: item
                .itunes_ext()
                .and_then(|itunes| itunes.keywords())
                .map(|k| k.split(',').collect::<Vec<_>>()),
            duration: item
                .itunes_ext()
                .and_then(|itunes| itunes.duration())
                .and_then(|d| parse_duration_from_str(d))
                .map(|x| x.num_seconds() as i64),
            show_notes: item
                .content()
                .or_else(|| item.itunes_ext().and_then(|itunes| itunes.summary())),
            url: item.link().and_then(|u| Url::parse(u).ok()),
            media_url: item
                .enclosure()
                .and_then(|e| Url::parse(e.url()).ok())
                .ok_or_else(|| anyhow::format_err!("field enclosure is required"))?,
            explicit: parse_explicit(item.itunes_ext()),
        })
    }
}
fn parse_explicit(it_ext: Option<&rss::extension::itunes::ITunesItemExtension>) -> bool {
    matches!(
        it_ext.and_then(|ext| ext.explicit()),
        Some("Yes") | Some("yes") | Some("true") | Some("True") | Some("explicit")
    )
}
