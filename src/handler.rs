use std::{borrow::Cow, fmt::Display, ops::Deref, path::Path, pin::Pin};

use crate::util::redirect;
use actix::fut::ready;
use actix_files::NamedFile;
use actix_web::http;
use actix_web::{web, Either, HttpRequest, HttpResponse};
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

#[derive(Debug)]
pub struct UrlPath<T: AsRef<str>> {
    inner: web::Path<T>,
}

impl<T: AsRef<str> + serde::de::DeserializeOwned> From<T> for UrlPath<T> {
    fn from(value: T) -> Self {
        Self {
            inner: web::Path::from(value),
        }
    }
}

impl<T: AsRef<str> + serde::de::DeserializeOwned> Deref for UrlPath<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AsRef<str> + serde::de::DeserializeOwned + 'static> actix_web::FromRequest for UrlPath<T> {
    type Config = web::PathConfig;

    type Error = actix_web::Error;

    type Future = Pin<Box<dyn futures_util::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        let path_ret = web::Path::from_request(req, payload);
        Box::pin(async move {
            match path_ret.await {
                Ok(path) => Ok(Self { inner: path }),
                Err(err) => Err(err),
            }
        })
    }
}

impl<T: AsRef<str> + serde::de::DeserializeOwned> UrlPath<T> {
    fn decode(&self) -> String {
        let path_ref = self.inner.as_ref();
        crate::util::percent_decode(path_ref.as_ref())
    }
}
