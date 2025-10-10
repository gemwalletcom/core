use std::collections::HashMap;

use serde::Deserialize;
use url::Url as UrlParser;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Url {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Override {
    pub rpc_method: Option<String>,
    pub path: Option<String>,
    pub url: String,
}

impl Url {
    pub fn host(&self) -> String {
        if let Ok(parsed_url) = UrlParser::parse(&self.url) {
            parsed_url.host_str().unwrap_or_default().to_string()
        } else {
            self.url.clone()
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub url: Url,
    pub block_number: u64,
    pub latency: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_url(url: &str) -> Url {
        Url {
            url: url.to_string(),
            headers: None,
        }
    }

    #[test]
    fn host_parsing() {
        assert_eq!(make_url("https://alpha.example.test/status").host(), "alpha.example.test");
        assert_eq!(make_url("http://127.0.0.1:8545").host(), "127.0.0.1");
        assert_eq!(make_url("rpc.provider.local").host(), "rpc.provider.local");
        assert_eq!(make_url("  https://example.com  ").host(), "example.com");
        assert_eq!(make_url("wss://node.example.com:443/ws").host(), "node.example.com");
        assert_eq!(make_url("https://fallback.example.com:8080/path").host(), "fallback.example.com");
    }
}
