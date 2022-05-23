extern crate reqwest;

use reqwest::{header, Method};

use crate::parser::HttpRequest;
use std::{str::FromStr, time::Duration};

#[derive(Default)]
pub struct HttpClient {
    pub client: reqwest::blocking::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(headers)
            .no_gzip()
            .build()
            .unwrap();
        HttpClient { client }
    }

    pub fn execute(&self, req: &HttpRequest) -> Result<(), Box<dyn std::error::Error>> {
        let request = self
            .client
            .request(Method::from_str(req.method.to_string().as_str())?, &req.url);

        let headers: header::HeaderMap = (&req.headers).try_into()?;

        let body = req.body.clone();
        let res = request.headers(headers).body(body).send()?;

        println!("{}\n", res.text()?);

        Ok(())
    }
}
