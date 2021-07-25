use http::{header, response::Builder, Response};
use hyper::Body;
use serde::Serialize;

#[derive(Debug)]
pub enum JsonError {
    Serialize(serde_json::Error),
    Http(http::Error),
}

pub trait ResponseExt {
    fn json<T>(self, json: &T) -> Result<Response<Body>, JsonError>
    where
        T: Serialize;
}

impl ResponseExt for Builder {
    fn json<T>(self, json: &T) -> Result<Response<Body>, JsonError>
    where
        T: Serialize,
    {
        serde_json::to_string(json)
            .map(Body::from)
            .map_err(JsonError::Serialize)
            .and_then(|body| {
                self.header(header::CONTENT_TYPE, "application/json")
                    .body(body)
                    .map_err(JsonError::Http)
            })
    }
}
