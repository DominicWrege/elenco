use std::convert::{TryFrom, TryInto};
use url::Url;

pub fn parse_img_url(feed: &rss::Channel) -> Option<Url> {
    feed.image()
        .and_then(|img| Url::parse(img.url()).ok())
        .or_else(|| {
            feed.itunes_ext()
                .and_then(|it| it.image().and_then(|u| Url::parse(u).ok()))
        })
}
pub fn parse_author(feed: &rss::Channel) -> String {
    feed.itunes_ext()
        .and_then(|x| x.author())
        .unwrap_or_default()
        .into()
}

pub fn episode_list<'a, T>(feed: &'a rss::Channel) -> Vec<T>
where
    T: TryFrom<&'a rss::Item>,
{
    feed.items()
        .iter()
        .flat_map(|item| item.try_into().ok())
        .collect()
}
