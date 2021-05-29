use std::{ops::Deref, pin::Pin};

use actix_web::{web, HttpRequest};

#[derive(Debug)]
pub struct Path<T: AsRef<str>> {
    inner: web::Path<T>,
}

impl<T: AsRef<str> + serde::de::DeserializeOwned> From<T> for Path<T> {
    fn from(value: T) -> Self {
        Self {
            inner: web::Path::from(value),
        }
    }
}

impl<T: AsRef<str>> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AsRef<str> + serde::de::DeserializeOwned + 'static> actix_web::FromRequest for Path<T> {
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

impl<T: AsRef<str> + serde::de::DeserializeOwned> Path<T> {
    pub fn decode(&self) -> String {
        let path_ref = self.inner.as_ref();
        crate::util::percent_decode(path_ref.as_ref())
    }
}
