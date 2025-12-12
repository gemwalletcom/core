use std::str::FromStr;

use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{Method, Request};

use super::request_url::RequestUrl;

pub struct RequestBuilder;

impl RequestBuilder {
    pub fn build(method: &Method, url: &RequestUrl, body: Vec<u8>, mut headers: HeaderMap) -> Result<Request, Box<dyn std::error::Error + Send + Sync>> {
        Self::apply_url_params(&mut headers, url);
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

    pub fn filter_headers(original_headers: &HeaderMap, forward_headers: &[HeaderName]) -> HeaderMap {
        original_headers
            .iter()
            .filter_map(|(k, v)| if forward_headers.contains(k) { Some((k.clone(), v.clone())) } else { None })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Url;
    use crate::proxy::constants::JSON_CONTENT_TYPE;
    use reqwest::Method as HttpMethod;
    use reqwest::header;

    fn make_request_url(base: &str, path: &str, header_kv: Option<(&str, &str)>) -> RequestUrl {
        let mut url = Url {
            url: base.to_string(),
            headers: None,
        };
        if let Some((k, v)) = header_kv {
            url.headers = Some({
                let mut m = std::collections::HashMap::new();
                m.insert(k.to_string(), v.to_string());
                m
            });
        }
        RequestUrl::from_parts(url, path)
    }

    #[test]
    fn test_build_with_headers() {
        let req_url = make_request_url("https://example.com", "/rpc", Some(("x-api-key", "secret")));
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));

        let req = RequestBuilder::build(&HttpMethod::POST, &req_url, b"{}".to_vec(), headers).expect("build");

        assert_eq!(req.method(), &HttpMethod::POST);
        assert_eq!(req.url().to_string(), "https://example.com/rpc");

        let headers = req.headers();
        assert_eq!(headers.get(header::CONTENT_TYPE).unwrap(), &header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        assert_eq!(headers.get("x-api-key").unwrap(), &header::HeaderValue::from_str("secret").unwrap());
    }

    #[test]
    fn test_filter_headers() {
        let mut orig_headers = HeaderMap::new();
        orig_headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static(JSON_CONTENT_TYPE));
        orig_headers.insert("x-drop", header::HeaderValue::from_static("dropme"));

        let keep = [header::CONTENT_TYPE];
        let filtered = RequestBuilder::filter_headers(&orig_headers, &keep);

        assert!(filtered.get("x-drop").is_none());
        assert_eq!(
            filtered.get(header::CONTENT_TYPE).unwrap(),
            &header::HeaderValue::from_static(JSON_CONTENT_TYPE)
        );
    }
}
