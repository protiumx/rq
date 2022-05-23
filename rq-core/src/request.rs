extern crate reqwest;

use reqwest::header;

use crate::parser::{HttpMethod, HttpRequest};
use std::time::Duration;

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
        let request = match req.method {
            HttpMethod::Get => self.client.get(&req.url),
            HttpMethod::Post => self.client.post(&req.url),
            HttpMethod::Put => self.client.post(&req.url),
            HttpMethod::Delete => self.client.post(&req.url),
        };

        let headers: header::HeaderMap = (&req.headers).try_into()?;

        let body = req.body.clone();
        let res = request.headers(headers).body(body).send()?;

        println!("{}\n", res.text()?);

        Ok(())
    }
}
