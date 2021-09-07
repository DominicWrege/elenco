pub mod auth;
pub mod author;
pub mod category;
pub mod comment;
pub mod episode;
pub mod error;
pub mod feed;
pub mod manage;
pub mod save_preview_feed;
pub mod subscription;
pub mod user;
use self::error::ApiError;
use tokio_pg_mapper::FromTokioPostgresRow;

pub type ApiJsonResult<T> = Result<Json<T>, ApiError>;

use crate::{
    inc_sql,
    model::{feed::Image, Meta},
    path::Path,
    util::{redirect, serialize},
    State,
};
use actix_files::NamedFile;
use actix_web::{
    web::{self, Json},
    Either, HttpRequest, HttpResponse,
};

pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let file_name = req.match_info().query("file_name");
    let folder = std::path::Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&file_name)) {
        Either::Left(file)
    } else {
        Either::Right(redirect("/404"))
    }
}

pub async fn image_for_feed(state: web::Data<State>, title: Path<String>) -> ApiJsonResult<Image> {
    let client = state.db_pool.get().await?;
    let stmnt = client.prepare(inc_sql!("get/feed_image")).await?;

    let row = client.query_one(&stmnt, &[&title.decode()]).await?;
    serialize(Image::from_row(row)?)
}

pub async fn meta(state: web::Data<State>) -> ApiJsonResult<Meta> {
    let client = state.db_pool.get().await?;
    let count_episodes = client
        .query_one(inc_sql!("get/meta/count_episode"), &[])
        .await?
        .get::<_, i64>(0);
    let episodes_duration = client
        .query_one(inc_sql!("get/meta/count_episode_duration"), &[])
        .await?
        .get::<_, i64>(0);
    let count_feeds = client
        .query_one(inc_sql!("get/meta/count_feed"), &[])
        .await?
        .get::<_, i64>(0);
    let count_authors = client
        .query_one(inc_sql!("get/meta/count_author"), &[])
        .await?
        .get::<_, i64>(0);
    serialize(Meta {
        episodes_duration,
        count_episodes,
        count_authors,
        count_feeds,
    })
}
