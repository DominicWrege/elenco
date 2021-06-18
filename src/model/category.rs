use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    id: i32,
    pub description: String,
    pub children: Vec<SubCategory>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PostgresMapper)]
#[serde(rename_all = "camelCase")]
#[pg_mapper(table = "subCategory")]
pub struct SubCategory {
    id: i32,
    pub description: String,
}

impl Category {
    pub fn from(row: &tokio_postgres::Row, children: Vec<SubCategory>) -> Self {
        let id: i32 = row.get("id");
        let description: String = row.get("description");
        Category {
            id,
            description,
            children,
        }
    }
}
