use actix_web::{http, HttpResponse};

pub fn redirect<P>(path: P) -> HttpResponse
where
    P: AsRef<str>,
    P: std::fmt::Display,
{
    HttpResponse::Found()
        .header(http::header::LOCATION, format!("{}", path))
        .finish()
}
