
pub mod author;
pub mod category;
pub mod episode;
pub mod error;
pub mod feed;
pub mod auth;
pub mod comment;
pub mod user;
pub mod subscription;
pub mod manage;
pub mod save_preview_feed;
use std::path::Path;

use self::error::ApiError;

pub type ApiJsonResult<T> = Result<Json<T>, ApiError>;



use actix_files::NamedFile;
use actix_web::{Either, HttpRequest, HttpResponse, web::{Json}};
use crate::util::redirect;


pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let file_name = req.match_info().query("file_name");
    let folder = Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&file_name)) {
        Either::Left(file)
    } else {
        Either::Right(redirect("/404"))
    }
}