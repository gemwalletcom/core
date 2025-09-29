use std::collections::HashMap;

use primitives::Chain;
use serde::Deserialize;
use url::Url as UrlParser;

use super::NodeMonitoringConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub domain: String,
    pub chain: Chain,
    pub block_delay: Option<u64>,
    pub poll_interval_seconds: Option<u64>,
    pub urls: Vec<Url>,
}

impl Domain {
    pub fn get_poll_interval_seconds(&self, monitoring_config: &NodeMonitoringConfig) -> u64 {
        self.poll_interval_seconds.unwrap_or(monitoring_config.poll_interval_seconds)
    }

    pub fn get_block_delay(&self, monitoring_config: &NodeMonitoringConfig) -> u64 {
        self.block_delay.unwrap_or(monitoring_config.block_delay)
    }

    pub fn is_url_behind(&self, url: Url, results: Vec<NodeResult>, monitoring_config: &NodeMonitoringConfig) -> bool {
        if let Some(index) = results.iter().position(|r| r.url == url) {
            let node = results[index].clone();
            if let Some(max_block_number) = Self::find_highest_block_number(results)
                && node.block_number + self.get_block_delay(monitoring_config) >= max_block_number.block_number
            {
                return false;
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

    #[test]
    fn host_strips_scheme_and_path() {
        let url = Url {
            url: "https://alpha.example.test/status".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "alpha.example.test");
    }

    #[test]
    fn host_drops_port() {
        let url = Url {
            url: "http://127.0.0.1:8545".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "127.0.0.1");
    }

    #[test]
    fn host_keeps_plain_value() {
        let url = Url {
            url: "rpc.provider.local".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "rpc.provider.local");
    }

    #[test]
    fn host_trims_whitespace() {
        let url = Url {
            url: "  https://example.com  ".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn host_handles_ws_scheme() {
        let url = Url {
            url: "wss://node.example.com:443/ws".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "node.example.com");
    }
    #[test]
    fn host_fallback_parsing() {
        let url = Url {
            url: "https://fallback.example.com:8080/path".to_string(),
            headers: None,
        };
        assert_eq!(url.host(), "fallback.example.com");
    }

    #[test]
    fn get_poll_interval_seconds_uses_domain_override() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: Some(20),
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 45,
            block_delay: 100,
        };

        assert_eq!(domain.get_poll_interval_seconds(&monitoring_config), 20);
    }

    #[test]
    fn get_poll_interval_seconds_uses_global_fallback() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 45,
            block_delay: 100,
        };

        assert_eq!(domain.get_poll_interval_seconds(&monitoring_config), 45);
    }

    #[test]
    fn get_poll_interval_seconds_uses_default_fallback() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 60 * 15,
            block_delay: 100,
        };

        assert_eq!(domain.get_poll_interval_seconds(&monitoring_config), 60 * 15);
    }

    #[test]
    fn get_block_delay_uses_domain_override() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: Some(50),
            poll_interval_seconds: None,
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 60 * 15,
            block_delay: 200,
        };

        assert_eq!(domain.get_block_delay(&monitoring_config), 50);
    }

    #[test]
    fn get_block_delay_uses_global_fallback() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 60 * 15,
            block_delay: 200,
        };

        assert_eq!(domain.get_block_delay(&monitoring_config), 200);
    }

    #[test]
    fn get_block_delay_uses_default_fallback() {
        use crate::config::NodeMonitoringConfig;

        let domain = Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            urls: vec![],
        };

        let monitoring_config = NodeMonitoringConfig {
            poll_interval_seconds: 60 * 15,
            block_delay: 100,
        };

        assert_eq!(domain.get_block_delay(&monitoring_config), 100);
    }
}
