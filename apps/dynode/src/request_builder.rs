use std::str::FromStr;

use bytes::Bytes;
use http_body_util::Full;
use hyper::header::{self, HeaderName};
use hyper::{HeaderMap, Method, Request};

use crate::request_url::RequestUrl;

const JSON_CONTENT_TYPE: &str = "application/json";
const JSON_HEADER: header::HeaderValue = header::HeaderValue::from_static(JSON_CONTENT_TYPE);

pub struct RequestBuilder;

impl RequestBuilder {
    pub fn build_jsonrpc(
        url: &RequestUrl,
        method: &Method,
        body: Bytes,
    ) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, JSON_HEADER.clone());
        Self::apply_url_params(&mut headers, url);
        Self::build(method, url, body, headers)
    }

    pub fn build_forwarded(
        method: &Method,
        url: &RequestUrl,
        body: Bytes,
        original_headers: &HeaderMap,
        keep_headers: &[HeaderName],
    ) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = Self::filter_headers(original_headers, keep_headers);
        Self::apply_url_params(&mut headers, url);
        Self::build(method, url, body, headers)
    }

    fn build(
        method: &Method,
        url: &RequestUrl,
        body: Bytes,
        headers: HeaderMap,
    ) -> Result<Request<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = Request::builder().method(method.clone()).uri(url.uri.clone()).body(Full::new(body))?;
        *request.headers_mut() = headers;
        Ok(request)
    }

    fn apply_url_params(headers: &mut HeaderMap, url: &RequestUrl) {
        for (key, value) in &url.params {
            if let (Ok(name), Ok(val)) = (HeaderName::from_str(key), value.parse()) {
                headers.append(name, val);
            }
        }
    }

    pub fn filter_headers(original_headers: &HeaderMap, keep_headers: &[HeaderName]) -> HeaderMap {
        original_headers
            .iter()
            .filter_map(|(k, v)| if keep_headers.contains(k) { Some((k.clone(), v.clone())) } else { None })
            .collect()
    }
}
