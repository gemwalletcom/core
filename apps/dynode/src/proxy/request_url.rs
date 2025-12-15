use std::collections::HashMap;

use reqwest::Url as ReqwestUrl;

use crate::config::Url;

#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub url: ReqwestUrl,
    pub params: HashMap<String, String>,
}

impl RequestUrl {
    pub fn from_parts(url: Url, original_path_and_query: &str) -> RequestUrl {
        let path = if original_path_and_query == "/" {
            "".to_string()
        } else {
            original_path_and_query.to_string()
        };
        let combined = format!("{}{}", url.url, path);
        let resolved = ReqwestUrl::parse(&combined).expect("invalid url");

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
        };
        let request_url = RequestUrl::from_parts(url, "/path");
        assert_eq!(request_url.url.to_string(), "https://example.com/path");
        assert!(request_url.params.is_empty());
    }

    #[test]
    fn test_from_uri_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), "secret".to_string());
        let url = Url {
            url: "https://example.com".to_string(),
            headers: Some(headers),
        };
        let request_url = RequestUrl::from_parts(url, "/path");
        assert_eq!(request_url.params.get("x-api-key"), Some(&"secret".to_string()));
    }
}
