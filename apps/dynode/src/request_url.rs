use std::{collections::HashMap, str::FromStr};

use hyper::Uri;

use crate::config::Url;

#[derive(Debug, Clone)]
pub struct RequestUrl {
    pub uri: Uri,
    pub params: HashMap<String, String>,
}

impl RequestUrl {
    pub fn from_uri(url: Url, url_override: HashMap<String, Url>, original_uri: &Uri) -> RequestUrl {
        let path = if original_uri.path() == "/" {
            String::new()
        } else {
            original_uri.to_string()
        };
        let uri = url.url + &path;
        let uri = uri.parse::<hyper::Uri>().expect("invalid url");

        for (path, endpoint) in url_override.clone() {
            if uri.path() == path {
                let uri = Uri::from_str(endpoint.url.clone().as_str()).unwrap();
                return RequestUrl {
                    uri,
                    params: endpoint.headers.unwrap_or_default(),
                };
            }
        }

        RequestUrl {
            uri,
            params: url.headers.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::HashMap, str::FromStr};

    #[test]
    fn test_from_uri() {
        let url = Url {
            url: "https://example.com".to_string(),
            headers: Some(HashMap::new()),
            urls_override: None,
        };
        let original_uri = Uri::from_str("/path").unwrap();
        let request_url = RequestUrl::from_uri(url.clone(), HashMap::new(), &original_uri);
        assert_eq!(request_url.uri.to_string(), "https://example.com/path");
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
        let request_url = RequestUrl::from_uri(url, urls_override, &original_uri);
        assert_eq!(request_url.uri.to_string(), "https://override.com/");
        assert_eq!(*request_url.params.get("key").unwrap(), "value".to_string());
    }
}
