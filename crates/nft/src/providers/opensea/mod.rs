pub mod client;
pub mod mapper;
pub mod model;
pub mod provider;

pub use client::OpenSeaClient;

use reqwest::header::{HeaderMap, HeaderValue};

pub fn create_client(api_key: &str) -> reqwest::Client {
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(api_key).unwrap());
    reqwest::Client::builder().default_headers(headers).build().unwrap()
}
