use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};
use tokio_pg_mapper_derive::PostgresMapper;
#[derive(Debug, PostgresMapper, Serialize)]
#[pg_mapper(table = "channel")]
pub struct Channel {
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
