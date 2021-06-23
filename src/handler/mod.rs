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
use std::path::Path;

use self::error::ApiError;

pub type ApiJsonResult<T> = Result<Json<T>, ApiError>;

use crate::util::redirect;
use actix_files::NamedFile;
use actix_web::{web::Json, Either, HttpRequest, HttpResponse};

pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let file_name = req.match_info().query("file_name");
    let folder = Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&file_name)) {
        Either::Left(file)
    } else {
        Either::Right(redirect("/404"))
    }
}
