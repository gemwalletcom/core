use bytes::Bytes;
use http_body_util::Full;
use hyper::{header, Response};

const JSON_CONTENT_TYPE: &str = "application/json";
const JSON_HEADER: header::HeaderValue = header::HeaderValue::from_static(JSON_CONTENT_TYPE);

pub struct ResponseBuilder;

impl ResponseBuilder {
    pub fn build_json<T: serde::Serialize>(data: &T) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let response_body = serde_json::to_vec(data)?;
        Self::build(Bytes::from(response_body), 200, JSON_CONTENT_TYPE)
    }

    pub fn build(data: Bytes, status: u16, content_type: &str) -> Result<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
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
        Ok(response)
    }

    pub fn build_cached(cached: crate::cache::CachedResponse) -> Response<Full<Bytes>> {
        let mut response = Response::new(Full::from(cached.body));

        if cached.status == 200 {
            *response.status_mut() = hyper::StatusCode::OK;
        } else {
            *response.status_mut() = hyper::StatusCode::from_u16(cached.status).unwrap_or(hyper::StatusCode::OK);
        }

        if cached.content_type == JSON_CONTENT_TYPE {
            response.headers_mut().insert(header::CONTENT_TYPE, JSON_HEADER.clone());
        } else {
            let header = cached.content_type.parse().unwrap_or(JSON_HEADER.clone());
            response.headers_mut().insert(header::CONTENT_TYPE, header);
        }

        response
    }
}
