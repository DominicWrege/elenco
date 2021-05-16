use std::path::Path;

use actix_files::NamedFile;
use actix_web::{Either, HttpRequest, HttpResponse};

use crate::util::redirect;

pub mod api;
pub mod auth;
pub mod feed_detail;
pub mod feed_preview;
pub mod general_error;
pub mod manage;
pub mod preview_error;
pub mod profile;
pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let file_name = req.match_info().query("file_name");
    let folder = Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&file_name)) {
        Either::Left(file)
    } else {
        Either::Right(redirect("/404"))
    }
}
