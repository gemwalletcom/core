use bytes::Bytes;
use http_body_util::Full;
use hyper::{header, HeaderMap, Response};

const JSON_CONTENT_TYPE: &str = "application/json";
const JSON_HEADER: header::HeaderValue = header::HeaderValue::from_static(JSON_CONTENT_TYPE);

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn create_upstream_headers(upstream_host: Option<&str>, latency: std::time::Duration) -> HeaderMap {
        let mut headers = HeaderMap::new();

        if let Some(host) = upstream_host {
            headers.insert("X-Upstream-Host", host.parse().unwrap_or_else(|_| "unknown".parse().unwrap()));
        }

        headers.insert("X-Upstream-Latency", format!("{}ms", latency.as_millis()).parse().unwrap());

        headers
    }
    pub fn build_with_headers(data: Bytes, status: u16, content_type: &str, additional_headers: HeaderMap) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut response = Response::new(Full::new(data));
        *response.status_mut() = hyper::StatusCode::from_u16(status).unwrap_or(hyper::StatusCode::OK);

        response.headers_mut().insert(
            header::CONTENT_TYPE,
            if content_type == JSON_CONTENT_TYPE {
                JSON_HEADER.clone()
            } else {
                content_type.parse().unwrap_or(JSON_HEADER.clone())
            },
        );

        response.headers_mut().extend(additional_headers);
        Ok(response)
    }

    pub fn build_cached_with_headers(cached: crate::cache::CachedResponse, additional_headers: HeaderMap) -> Response<Full<Bytes>> {
        let mut response = Response::new(Full::from(cached.body));

        *response.status_mut() = if cached.status == 200 {
            hyper::StatusCode::OK
        } else {
            hyper::StatusCode::from_u16(cached.status).unwrap_or(hyper::StatusCode::OK)
        };

        let content_header = if cached.content_type == JSON_CONTENT_TYPE {
            JSON_HEADER.clone()
        } else {
            cached.content_type.parse().unwrap_or(JSON_HEADER.clone())
        };
        response.headers_mut().insert(header::CONTENT_TYPE, content_header);

        response.headers_mut().extend(additional_headers);
        response
    }
}
