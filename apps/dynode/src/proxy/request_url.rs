use std::collections::HashMap;

use reqwest::Url as ReqwestUrl;

use crate::config::Url;

#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub url: ReqwestUrl,
    pub params: HashMap<String, String>,
}

impl RequestUrl {
    pub fn from_parts(url: Url, url_override: HashMap<String, Url>, original_path_and_query: &str) -> RequestUrl {
        let path = if original_path_and_query == "/" {
            "".to_string()
        } else {
            original_path_and_query.to_string()
        };
        let combined = format!("{}{}", url.url, path);
        let resolved = ReqwestUrl::parse(&combined).expect("invalid url");

        for (override_path, endpoint) in url_override {
            if resolved.path() == override_path {
                let override_url = ReqwestUrl::parse(&endpoint.url).expect("invalid override url");
                return RequestUrl {
                    url: override_url,
                    params: endpoint.headers.unwrap_or_default(),
                };
            }
        }

        RequestUrl {
            url: resolved,
            params: url.headers.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_uri() {
        let url = Url {
            url: "https://example.com".to_string(),
            headers: Some(HashMap::new()),
            urls_override: None,
        };
        let request_url = RequestUrl::from_parts(url.clone(), HashMap::new(), "/path");
        assert_eq!(request_url.url.to_string(), "https://example.com/path");
        assert!(request_url.params.is_empty());

        let mut urls_override = HashMap::new();
        urls_override.insert(
            "/path".to_string(),
            Url {
                url: "https://override.com/".to_string(),
                headers: Some({
                    let mut params = HashMap::new();
                    params.insert("key".to_string(), "value".to_string());
                    params
                }),
                urls_override: None,
            },
        );
        let request_url = RequestUrl::from_parts(url, urls_override, "/path");
        assert_eq!(request_url.url.to_string(), "https://override.com/");
        assert_eq!(*request_url.params.get("key").unwrap(), "value".to_string());
    }
}
