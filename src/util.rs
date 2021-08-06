use actix_web::{http, web::Json, HttpResponse};
use isolang::Language;
use percent_encoding::percent_decode_str;

use crate::{handler::error::ApiError, model::preview::feed::FeedPreview};

pub fn redirect<P>(path: P) -> HttpResponse
where
    P: AsRef<str>,
    P: std::fmt::Display,
{
    HttpResponse::Found()
        .append_header((http::header::LOCATION, format!("/web{}", path)))
        .finish()
}

pub fn page_not_found() -> HttpResponse {
    redirect("/404")
}

impl LanguageCodeLookup for FeedPreview<'_> {
    fn language_code(&self) -> Option<&str> {
        self.language
    }
}

pub trait LanguageCodeLookup {
    fn language_lookup(&self) -> Option<Language> {
        self.language_code()
            .and_then(|code| Language::from_639_1(code).or_else(|| Language::from_639_3(code)))
    }
    fn language_code(&self) -> Option<&str>;
}

pub fn percent_decode(text: &str) -> String {
    percent_decode_str(text).decode_utf8_lossy().to_string()
}

pub fn serialize<D>(data: D) -> Result<actix_web::web::Json<D>, ApiError> {
    Ok(Json(data))
}
