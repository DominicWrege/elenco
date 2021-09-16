use crate::model::category::Category;
use crate::path;
use crate::util::serialize;
use crate::{db, State};
use actix_web::web;

use super::ApiJsonResult;

pub async fn all(state: web::Data<State>) -> ApiJsonResult<Vec<Category>> {
    let client = state.db_pool.get().await?;
    serialize(db::category::get_all(&client).await?)
}

pub async fn by_id_or_name(
    state: web::Data<State>,
    path: path::Path<String>,
) -> ApiJsonResult<Category> {
    let client = state.db_pool.get().await?;
    let catagories = db::category::get_by_id_or_name(&client, &path.decode()).await?;
    serialize(catagories)
}
