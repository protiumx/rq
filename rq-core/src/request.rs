extern crate reqwest;

pub use reqwest::StatusCode;
use reqwest::{
    header::{self, HeaderMap},
    Client, Method,
};

use crate::parser::HttpRequest;
use std::{str::FromStr, time::Duration};

fn new_client() -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/json"),
    );

    Client::builder()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .no_gzip()
        .build()
        .unwrap()
}

pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub version: String,
    pub body: String,
}

impl Response {
    async fn from_reqwest(value: reqwest::Response) -> Self {
        let status = value.status();
        let version = format!("{:?}", value.version());
        let headers = value.headers().clone();
        let body = match value.text().await {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };

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
    let request =
        new_client().request(Method::from_str(req.method.to_string().as_str())?, &req.url);

    let headers: header::HeaderMap = (req.headers()).try_into()?;

    let body = req.body.clone();
    let res = request.headers(headers).body(body).send().await?;

    Ok(Response::from_reqwest(res).await)
}
