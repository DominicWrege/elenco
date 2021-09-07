pub mod error;
pub mod preview;
pub mod save;

use actix_web::web;
use url::Url;

use self::error::PreviewSaveError;

async fn fetch(url: &Url) -> Result<web::Bytes, PreviewSaveError> {
    let bytes = reqwest::get(url.clone())
        .await
        .map_err(|_err| PreviewSaveError::Fetch(url.clone()))?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(bytes)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedForm {
    pub feed_url: Url,
}
