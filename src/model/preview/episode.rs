use crate::time_date::{
    parse_datetime_rfc822, parse_duration_from_str, serialize_option_datetime, DurationFormator,
};
use chrono::offset::Utc;
use chrono::DateTime;
use reqwest::Url;
use serde::Serializer;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Episode {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    #[serde(default, serialize_with = "serialize_option_datetime")]
    pub published: Option<DateTime<Utc>>,
    pub keywords: Option<Vec<String>>,
    pub duration: Option<i64>,
    pub show_notes: Option<String>,
    pub web_link: Option<Url>,
    pub enclosure: MyEnclosure,
    pub explicit: bool,
    pub guid: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpisodeNext {
    pub items: Vec<Episode>,
    pub offset: Option<i64>,
}

impl TryFrom<&rss::Enclosure> for MyEnclosure {
    type Error = anyhow::Error;

    fn try_from(value: &rss::Enclosure) -> Result<Self, Self::Error> {
        Ok(Self {
            media_url: Url::parse(value.url())?,
            length: value.length().parse::<i64>().unwrap_or_default(),
            mime_type: value
                .mime_type()
                .parse()
                .unwrap_or(mime::Mime::from("audio/mpeg".parse().unwrap())),
        })
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyEnclosure {
    pub media_url: Url,
    pub length: i64,
    #[serde(serialize_with = "serialize_mime")]
    pub mime_type: mime::Mime,
}

pub fn serialize_mime<S>(mime: &mime::Mime, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&mime.to_string())
}

impl Episode {
    pub fn url(&self) -> Option<&str> {
        self.web_link.as_ref().map(|url| url.as_str())
    }

    pub fn media_url(&self) -> &str {
        self.enclosure.media_url.as_str()
    }

    pub fn from_items(items: &[rss::Item]) -> Vec<Episode> {
        let mut items: Vec<Episode> = items
            .iter()
            .flat_map(|item| {
                let ret = item.try_into();
                if ret.is_err() {
                    log::warn!("{:?}", ret);
                }
                ret
            })
            .collect();

        items.sort_by(|a, b| b.published.cmp(&a.published));
        items
    }
}

impl DurationFormator for Episode {
    fn duration(&self) -> Option<i64> {
        self.duration
    }
}

impl<'a> TryFrom<&'a rss::Item> for Episode {
    type Error = anyhow::Error;

    fn try_from(item: &'a rss::Item) -> Result<Self, Self::Error> {
        Ok(Self {
            id: 0,
            title: item
                .title()
                .map(|t| t.to_owned())
                .ok_or_else(|| anyhow::format_err!("field title is required"))?,
            description: parse_description(&item),
            published: item.pub_date().and_then(|d| parse_datetime_rfc822(d).ok()),
            keywords: item
                .itunes_ext()
                .and_then(|itunes| itunes.keywords())
                .map(|k| k.split(',').map(|k| k.to_owned()).collect::<Vec<_>>()),
            duration: item
                .itunes_ext()
                .and_then(|itunes| itunes.duration())
                .and_then(|d| parse_duration_from_str(d))
                .map(|x| x.num_seconds() as i64),
            show_notes: parse_show_notes(item),
            web_link: item.link().and_then(|u| Url::parse(u).ok()),
            explicit: parse_explicit(item.itunes_ext()),
            guid: item.guid().map(|guid| guid.value().to_string()),
            enclosure: item
                .enclosure()
                .and_then(|en| en.try_into().ok())
                .ok_or_else(|| {
                    anyhow::format_err!("field enclosure is not present or bad format")
                })?,
        })
    }
}

impl From<tokio_postgres::Row> for Episode {
    fn from(row: tokio_postgres::Row) -> Self {
        let media_url: String = row.get("media_url");
        let mime_type: String = row.get("mime_type");

        Self {
            id: row.get("id"),
            title: row.get("title"),
            description: row.get("description"),
            explicit: row.get("explicit"),
            duration: row.get::<_, Option<i64>>("duration"),
            web_link: row
                .get::<_, Option<String>>("web_link")
                .and_then(|url| Url::parse(&url).ok()),
            show_notes: row.get("show_notes"),
            enclosure: MyEnclosure {
                media_url: Url::parse(&media_url).unwrap(),
                length: row.get("media_length"),
                mime_type: mime::Mime::from_str(&mime_type).unwrap(),
            },
            published: row.get("published"),
            keywords: row.get("keywords"),
            guid: row.get("guid"),
        }
    }
}

fn parse_description(item: &rss::Item) -> Option<String> {
    let description = item
        .itunes_ext()
        .and_then(|itun| itun.summary())
        .or_else(|| item.description());

    description.map(|desc| {
        desc.split_whitespace()
            .take(52)
            .collect::<Vec<_>>()
            .join(" ")
    })
}

fn sanitize_html(html: &str) -> String {
    use ammonia::{Builder, UrlRelative};
    Builder::default()
        .link_rel(None)
        .url_relative(UrlRelative::Deny)
        .rm_tags(&[
            "img",
            "hr",
            "figure",
            "figcaption",
            "mark",
            "ruby",
            "iframe",
        ])
        .clean(html)
        .to_string()
}

fn parse_show_notes(item: &rss::Item) -> Option<String> {
    let s = (
        item.content(),
        item.itunes_ext().and_then(|it| it.summary()),
    );

    let show_notes = match s {
        (None, Some(summary)) => {
            let ret = match item.description() {
                Some(desc) if desc.len() > desc.len() => desc,
                _ => summary,
            };
            Some(ret)
        }
        (Some(content), None) => {
            let ret = match item.description() {
                Some(desc) if desc.len() > content.len() => desc,
                _ => content,
            };
            Some(ret)
        }
        (Some(content), Some(summary)) => {
            let ret = if content.len() > summary.len() {
                content
            } else {
                summary
            };
            Some(ret)
        }
        _ => item.description(),
    };

    show_notes.map(|notes| sanitize_html(notes))
}

fn parse_explicit(it_ext: Option<&rss::extension::itunes::ITunesItemExtension>) -> bool {
    matches!(
        it_ext.and_then(|ext| ext.explicit()),
        Some("Yes") | Some("yes") | Some("true") | Some("True") | Some("explicit")
    )
}
