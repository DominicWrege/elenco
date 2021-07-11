use actix_web::{
    web::{self, Json},
    HttpResponse,
};

use crate::{
    db::subscription,
    model::{user::Account, FeedUserMeta},
    State,
};

use super::{error::ApiError, ApiJsonResult};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribePayload {
    feed_id: i32,
}

pub async fn subscribe(
    state: web::Data<State>,
    json: Json<SubscribePayload>,
    session: actix_session::Session,
) -> Result<HttpResponse, ApiError> {
    let mut client = state.db_pool.get().await?;
    let acount = Account::from_session(&session).unwrap();
    subscription::subscribe(&mut client, acount.id(), json.feed_id).await?;

    Ok(HttpResponse::Created().finish())
}

pub async fn unsubscribe(
    state: web::Data<State>,
    json: Json<SubscribePayload>,
    session: actix_session::Session,
) -> Result<HttpResponse, ApiError> {
    let mut client = state.db_pool.get().await?;
    let acount = Account::from_session(&session).unwrap();
    subscription::unsubscribe(&mut client, acount.id(), json.feed_id).await?;

    Ok(HttpResponse::Ok().finish())
}

pub async fn subscription_info(
    state: web::Data<State>,
    json: Json<SubscribePayload>,
    session: actix_session::Session,
) -> ApiJsonResult<FeedUserMeta> {
    let client = state.db_pool.get().await?;
    let acount = Account::from_session(&session).unwrap();
    let resp_json =
        subscription::user_subscription_info(&client, acount.id(), json.feed_id).await?;
    Ok(Json(resp_json))
}
