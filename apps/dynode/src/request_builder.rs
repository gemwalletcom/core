use std::str::FromStr;

use bytes::Bytes;
use reqwest::header::{self, HeaderMap, HeaderName};
use reqwest::{Method, Request};

use crate::constants::JSON_HEADER;
use crate::request_url::RequestUrl;

pub struct RequestBuilder;

impl RequestBuilder {
    pub fn build_jsonrpc(url: &RequestUrl, method: &Method, body: Bytes) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
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
    ) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
        let mut headers = Self::filter_headers(original_headers, keep_headers);
        Self::apply_url_params(&mut headers, url);
        Self::build(method, url, body, headers)
    }

    fn build(method: &Method, url: &RequestUrl, body: Bytes, headers: HeaderMap) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
        let mut request = Request::new(method.clone(), url.url.clone());
        *request.headers_mut() = headers;
        *request.body_mut() = Some(body.into());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Url;
    use reqwest::Method as HttpMethod;

    fn make_request_url(base: &str, path: &str, header_kv: Option<(&str, &str)>) -> RequestUrl {
        let mut url = Url {
            url: base.to_string(),
            headers: None,
            urls_override: None,
        };
        if let Some((k, v)) = header_kv {
            url.headers = Some({
                let mut m = std::collections::HashMap::new();
                m.insert(k.to_string(), v.to_string());
                m
            });
        }
        RequestUrl::from_parts(url, std::collections::HashMap::new(), path)
    }

    #[test]
    fn test_build_jsonrpc_sets_headers_and_uri() {
        let req_url = make_request_url("https://example.com", "/rpc", Some(("x-api-key", "secret")));
        let req = RequestBuilder::build_jsonrpc(&req_url, &HttpMethod::POST, Bytes::from("{}".as_bytes().to_vec())).expect("build_jsonrpc");

        assert_eq!(req.method(), &HttpMethod::POST);
        assert_eq!(req.url().to_string(), "https://example.com/rpc");

        let headers = req.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            &header::HeaderValue::from_static("application/json")
        );
        assert_eq!(headers.get("x-api-key").unwrap(), &header::HeaderValue::from_str("secret").unwrap());
    }

    #[test]
    fn test_build_forwarded_filters_and_applies_params() {
        let req_url = make_request_url("https://example.com", "/data", Some(("x-api-key", "k")));

        let mut orig_headers = HeaderMap::new();
        orig_headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        orig_headers.insert("x-drop", header::HeaderValue::from_static("dropme"));

        let keep = [header::CONTENT_TYPE];
        let req = RequestBuilder::build_forwarded(&HttpMethod::GET, &req_url, Bytes::from_static(b""), &orig_headers, &keep).expect("build_forwarded");

        assert_eq!(req.method(), &HttpMethod::GET);
        assert_eq!(req.url().to_string(), "https://example.com/data");
        let headers = req.headers();
        assert!(headers.get("x-drop").is_none());
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            &header::HeaderValue::from_static("application/json")
        );
        assert_eq!(headers.get("x-api-key").unwrap(), &header::HeaderValue::from_static("k"));
    }
}
