extern crate reqwest;

use bytes::Bytes;
use once_cell::sync::Lazy;
pub use reqwest::StatusCode;
use reqwest::{header::HeaderMap, Client};

use crate::parser::HttpRequest;
use std::{fmt::Display, time::Duration};

static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .no_gzip()
        .build()
        .unwrap()
});

#[derive(Clone)]
pub enum Content {
    Bytes(Bytes),
    Text(String),
}

impl Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Bytes(_) => write!(f, "<raw bytes>"),
            Content::Text(s) => write!(f, "{s}"),
        }
    }
}

impl From<Bytes> for Content {
    fn from(value: Bytes) -> Self {
        match String::from_utf8(value.clone().into()) {
            Ok(s) => Content::Text(s),
            Err(_) => Content::Bytes(value),
        }
    }
}

#[derive(Clone)]
pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub version: String,
    pub body: Content,
}

impl Response {
    async fn from_reqwest(value: reqwest::Response) -> Self {
        let status = value.status();
        let version = format!("{:?}", value.version());
        let headers = value.headers().clone();
        let body = value.bytes().await.unwrap().into();

        Self {
            status,
            version,
            headers,
            body,
        }
    }
}

type RequestResult = Result<Response, Box<dyn std::error::Error + Send + Sync>>;

pub async fn execute(req: &HttpRequest) -> RequestResult {
    let request = CLIENT
        .request(req.method.clone(), &req.url)
        .headers(req.headers())
        .body(req.body.clone());

    let res = request.send().await?;

    Ok(Response::from_reqwest(res).await)
}
