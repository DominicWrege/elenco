use std::path::Path;

use actix_files::NamedFile;
use actix_web::{Either, HttpRequest, HttpResponse};

use crate::util::redirect;

pub mod api;
pub mod auth;
pub mod feed_detail;
pub mod general_error;
pub mod moderator;
pub mod feed_preview;
pub mod profile;

pub async fn serve_img(req: HttpRequest) -> Either<NamedFile, HttpResponse> {
    let filename = req.match_info().query("filename");
    let folder = Path::new("./img-cache");
    if let Ok(file) = NamedFile::open(folder.join(&filename)) {
        Either::A(file)
    } else {
        Either::B(redirect("/404"))
    }
}
