use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};

use super::constants::{JSON_CONTENT_TYPE, JSON_HEADER};

const X_UPSTREAM_HOST: HeaderName = HeaderName::from_static("x-upstream-host");
const X_UPSTREAM_LATENCY: HeaderName = HeaderName::from_static("x-upstream-latency");

#[derive(Debug, Clone)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
}

impl ProxyResponse {
    pub fn new(status: u16, headers: HeaderMap, body: Vec<u8>) -> Self {
        Self { status, headers, body }
    }
}

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn create_upstream_headers(upstream_host: Option<&str>, latency: std::time::Duration) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if let Some(host) = upstream_host {
            headers.insert(
                X_UPSTREAM_HOST,
                HeaderValue::from_str(host).unwrap_or_else(|_| HeaderValue::from_static("unknown")),
            );
        }

        headers.insert(
            X_UPSTREAM_LATENCY,
            HeaderValue::from_str(&format!("{}ms", latency.as_millis())).unwrap_or_else(|_| HeaderValue::from_static("0ms")),
        );

        headers
    }

    pub fn build_with_headers(
        data: Vec<u8>,
        status: u16,
        content_type: &str,
        additional_headers: HeaderMap,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();

        let content_header = if content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            HeaderValue::from_str(content_type).unwrap_or_else(|_| JSON_HEADER.clone())
        };

        headers.insert(header::CONTENT_TYPE, content_header);
        headers.extend(additional_headers);

        Ok(ProxyResponse::new(status, headers, data))
    }

    pub fn build_cached_with_headers(cached: crate::cache::CachedResponse, additional_headers: HeaderMap) -> ProxyResponse {
        let mut headers = HeaderMap::new();

        let content_header = if cached.content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            HeaderValue::from_str(&cached.content_type).unwrap_or_else(|_| JSON_HEADER.clone())
        };

        headers.insert(header::CONTENT_TYPE, content_header);
        headers.extend(additional_headers);

        ProxyResponse::new(cached.status, headers, cached.body)
    }

    pub fn build_json_response_with_headers<T: serde::Serialize>(
        data: &T,
        headers: HeaderMap,
    ) -> Result<ProxyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let response_body = serde_json::to_vec(data)?;
        Self::build_with_headers(response_body, 200, JSON_CONTENT_TYPE, headers)
    }
}
