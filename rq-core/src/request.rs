extern crate reqwest;

use reqwest::{header, Client, Method};

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

pub async fn execute(
    req: &HttpRequest,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let request =
        new_client().request(Method::from_str(req.method.to_string().as_str())?, &req.url);

    let headers: header::HeaderMap = (&req.headers).try_into()?;

    let body = req.body.clone();
    let res = request.headers(headers).body(body).send().await?;

    let content = res.text().await?;
    Ok(content)
}
