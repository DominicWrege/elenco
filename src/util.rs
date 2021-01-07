use actix_web::{http, HttpResponse};

use crate::model::channel::RawFeed;
use isolang::Language;
pub fn redirect<P>(path: P) -> HttpResponse
where
    P: AsRef<str>,
    P: std::fmt::Display,
{
    HttpResponse::Found()
        .header(http::header::LOCATION, format!("/web{}", path))
        .finish()
}

pub fn page_not_found() -> HttpResponse {
    redirect("/404")
}

impl LanguageCodeLookup for RawFeed<'_> {
    fn language_code(&self) -> Option<&str> {
        self.language_code
    }
}

pub trait LanguageCodeLookup {
    fn language_lookup(&self) -> Option<Language> {
        self.language_code().and_then(|code| {
            Language::from_639_1(code)
                .or_else(|| Language::from_639_3(code))
        })
    }
    fn language_code(&self) -> Option<&str>;
}
