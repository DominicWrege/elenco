use std::path::Path;

use crate::util::redirect;
use actix_files::NamedFile;

use actix_web::{Either, HttpRequest, HttpResponse};
pub mod api;
pub mod auth;
pub mod feed_detail;
pub mod manage;
pub mod save_preview_feed;
pub mod subscription;
pub mod user_feed;

pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let file_name = req.match_info().query("file_name");
    let folder = Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&file_name)) {
        Either::Left(file)
    } else {
        Either::Right(redirect("/404"))
    }
}
