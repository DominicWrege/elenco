use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
#[derive(Debug, Serialize)]
pub struct FeedJson<'a> {
    pub id: i32,
    pub url: String,
    pub title: String,
    pub author: String,
    pub img: Option<String>,
    pub link_web: String,
    pub description: String,
    pub subtitle: Option<String>,
    pub language: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub last_modified: DateTime<Utc>,
    pub categories: &'a [Category],
}

pub fn serialize_datetime<S>(date: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    return s.serialize_str(&date.format("%d.%m.%g").to_string());
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    id: i32,
    description: String,
    childreen: Vec<SubCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubCategory {
    id: i32,
    description: String,
}

impl<'a> FeedJson<'a> {
    pub fn from(row: &tokio_postgres::Row, categories: &'a [Category]) -> Self {
        Self {
            id: row.get("id"),
            url: row.get("url"),
            title: row.get("title"),
            author: row.get("author"),
            img: row.get("img"),
            link_web: row.get("link_web"),
            description: row.get("description"),
            subtitle: row.get("subtitle"),
            language: row.get("language"),
            last_modified: row.get("last_modified"),
            categories,
        }
    }
}

impl From<tokio_postgres::Row> for Category {
    fn from(row: tokio_postgres::Row) -> Self {
        let id: i32 = row.get(0);
        let description: String = row.get(1);
        Category {
            description,
            id,
            childreen: serde_json::from_value(row.get(2)).unwrap(),
        }
    }
}
