use crate::handler::api::ApiError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use tokio_postgres::Client;
#[derive(Debug, Serialize)]
pub struct FeedJson {
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
    pub categories: Vec<Category>,
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

impl FeedJson {
    pub fn from(row: &tokio_postgres::Row, categories: Vec<Category>) -> Self {
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
    pub async fn from2(client: &Client, row: tokio_postgres::Row) -> Result<FeedJson, ApiError> {
        use crate::db::categories_for_feed;
        let category_id = row.get("id");
        let categories = categories_for_feed(&client, category_id).await?;
        Ok(FeedJson::from(&row, categories))
    }
}

impl From<tokio_postgres::Row> for Category {
    fn from(row: tokio_postgres::Row) -> Self {
        let id: i32 = row.get("id");
        let description: String = row.get("description");
        Category {
            description,
            id,
            childreen: serde_json::from_value(row.get("subcategories")).unwrap(),
        }
    }
}
