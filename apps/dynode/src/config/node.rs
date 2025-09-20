use std::collections::HashMap;

use primitives::Chain;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub domain: String,
    pub chain: Chain,
    pub block_delay: Option<u64>,
    pub poll_interval_seconds: Option<u64>,
    pub urls: Vec<Url>,
}

impl Domain {
    pub fn get_poll_interval_seconds(&self) -> u64 {
        self.poll_interval_seconds.unwrap_or(600)
    }

    pub fn get_block_delay(&self) -> u64 {
        self.block_delay.unwrap_or(100)
    }

    pub fn is_url_behind(&self, url: Url, results: Vec<NodeResult>) -> bool {
        if let Some(index) = results.iter().position(|r| r.url == url) {
            let node = results[index].clone();
            if let Some(max_block_number) = Self::find_highest_block_number(results) {
                if node.block_number + self.get_block_delay() >= max_block_number.block_number {
                    return false;
                }
            }
        }
        true
    }

    pub fn find_highest_block_number(results: Vec<NodeResult>) -> Option<NodeResult> {
        results.into_iter().max_by(|x, y| x.block_number.cmp(&y.block_number))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Url {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub urls_override: Option<HashMap<String, Url>>,
}

impl Url {
    pub fn host(&self) -> String {
        let trimmed = self.url.trim();
        let mut value = match trimmed.find("://") {
            Some(index) => &trimmed[index + 3..],
            None => trimmed,
        };

        value = value.trim_start_matches('/');
        let host_port = value.split(|c| c == '/' || c == '?' || c == '#').next().unwrap_or("");

        if host_port.is_empty() {
            return trimmed.to_string();
        }

        if host_port.starts_with('[') {
            if let Some(end) = host_port.find(']') {
                return host_port[1..end].to_string();
            }
        }

        host_port
            .split_once(':')
            .map(|(host, _)| host.to_string())
            .unwrap_or_else(|| host_port.to_string())
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
    use super::Url;

    #[test]
    fn host_strips_scheme_and_path() {
        let url = Url {
            url: "https://alpha.example.test/status".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "alpha.example.test");
    }

    #[test]
    fn host_drops_port() {
        let url = Url {
            url: "http://127.0.0.1:8545".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "127.0.0.1");
    }

    #[test]
    fn host_keeps_plain_value() {
        let url = Url {
            url: "rpc.provider.local".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "rpc.provider.local");
    }

    #[test]
    fn host_trims_whitespace() {
        let url = Url {
            url: "  https://example.com  ".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn host_handles_ws_scheme() {
        let url = Url {
            url: "wss://node.example.com:443/ws".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "node.example.com");
    }

    #[test]
    fn host_handles_ipv6() {
        let url = Url {
            url: "https://[2001:db8::1]:8080/path".to_string(),
            headers: None,
            urls_override: None,
        };
        assert_eq!(url.host(), "2001:db8::1");
    }
}
