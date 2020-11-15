use crate::{
    db::{fetch_feeds, Feed},
    State,
};
use actix_web::{web, web::Json};

pub async fn all_feeds(state: web::Data<State>) -> Result<Json<Vec<Feed>>, ()> {
    Ok(Json(fetch_feeds(&state.db_pool).await.unwrap()))
}
