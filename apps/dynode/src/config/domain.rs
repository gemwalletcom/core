use primitives::Chain;
use serde::Deserialize;

use super::NodeMonitoringConfig;
use super::url::{NodeResult, Override, Url};

#[derive(Debug, Clone, Deserialize)]
pub struct Domain {
    pub domain: String,
    pub chain: Chain,
    pub block_delay: Option<u64>,
    pub poll_interval_seconds: Option<u64>,
    pub overrides: Option<Vec<Override>>,
    pub urls: Vec<Url>,
}

impl Domain {
    pub fn get_poll_interval_seconds(&self, monitoring_config: &NodeMonitoringConfig) -> u64 {
        self.poll_interval_seconds.unwrap_or(monitoring_config.poll_interval_seconds)
    }

    pub fn get_block_delay(&self, monitoring_config: &NodeMonitoringConfig) -> u64 {
        self.block_delay.unwrap_or(monitoring_config.block_delay)
    }

    pub fn resolve_url(&self, base_url: &Url, rpc_method: Option<&str>, request_path: Option<&str>) -> Url {
        let Some(overrides) = &self.overrides else {
            return base_url.clone();
        };

        for override_config in overrides {
            let rpc_matches = override_config
                .rpc_method
                .as_ref()
                .map_or(true, |override_method| Some(override_method.as_str()) == rpc_method);

            let path_matches = override_config
                .path
                .as_ref()
                .map_or(true, |override_path| Some(override_path.as_str()) == request_path);

            if rpc_matches && path_matches {
                return Url {
                    url: override_config.url.clone(),
                    headers: base_url.headers.clone(),
                };
            }
        }

        base_url.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_domain(poll_interval: Option<u64>, block_delay: Option<u64>) -> Domain {
        Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay,
            poll_interval_seconds: poll_interval,
            overrides: None,
            urls: vec![],
        }
    }

    fn make_url(url: &str) -> Url {
        Url {
            url: url.to_string(),
            headers: None,
        }
    }

    fn make_domain_with_overrides(overrides: Vec<Override>) -> Domain {
        Domain {
            domain: "test".to_string(),
            chain: primitives::Chain::Ethereum,
            block_delay: None,
            poll_interval_seconds: None,
            overrides: Some(overrides),
            urls: vec![make_url("https://example.com")],
        }
    }

    fn make_monitoring_config(poll_interval: u64, block_delay: u64) -> NodeMonitoringConfig {
        NodeMonitoringConfig {
            poll_interval_seconds: poll_interval,
            block_delay,
        }
    }

    #[test]
    fn get_poll_interval_seconds_uses_domain_override() {
        let domain = make_domain(Some(20), None);
        let config = make_monitoring_config(45, 100);
        assert_eq!(domain.get_poll_interval_seconds(&config), 20);
    }

    #[test]
    fn get_poll_interval_seconds_uses_global_fallback() {
        let domain = make_domain(None, None);
        let config = make_monitoring_config(45, 100);
        assert_eq!(domain.get_poll_interval_seconds(&config), 45);
    }

    #[test]
    fn get_block_delay_uses_domain_override() {
        let domain = make_domain(None, Some(50));
        let config = make_monitoring_config(900, 200);
        assert_eq!(domain.get_block_delay(&config), 50);
    }

    #[test]
    fn get_block_delay_uses_global_fallback() {
        let domain = make_domain(None, None);
        let config = make_monitoring_config(900, 200);
        assert_eq!(domain.get_block_delay(&config), 200);
    }

    #[test]
    fn resolve_url_without_override() {
        let domain = make_domain(None, None);
        let base_url = make_url("https://example.com/rpc");
        assert_eq!(domain.resolve_url(&base_url, Some("eth_sendTransaction"), None).url, "https://example.com/rpc");
    }

    #[test]
    fn resolve_url_with_rpc_method_override() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: Some("eth_sendTransaction".to_string()),
            path: None,
            url: "https://tx-relay.example.com".to_string(),
        }]);
        let base_url = make_url("https://example.com/rpc");
        assert_eq!(
            domain.resolve_url(&base_url, Some("eth_sendTransaction"), None).url,
            "https://tx-relay.example.com"
        );
    }

    #[test]
    fn resolve_url_with_rpc_method_and_path_override() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: Some("eth_sendTransaction".to_string()),
            path: None,
            url: "https://tx-relay.example.com/tx/submit".to_string(),
        }]);
        let base_url = make_url("https://example.com/rpc");
        assert_eq!(
            domain.resolve_url(&base_url, Some("eth_sendTransaction"), None).url,
            "https://tx-relay.example.com/tx/submit"
        );
    }

    #[test]
    fn resolve_url_without_matching_override() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: Some("eth_sendTransaction".to_string()),
            path: None,
            url: "https://tx-relay.example.com".to_string(),
        }]);
        let base_url = make_url("https://example.com/rpc");
        assert_eq!(domain.resolve_url(&base_url, Some("eth_blockNumber"), None).url, "https://example.com/rpc");
    }

    #[test]
    fn resolve_url_with_wildcard_override() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: None,
            path: None,
            url: "https://fallback.example.com/v2/rpc".to_string(),
        }]);
        let base_url = make_url("https://example.com/rpc");
        assert_eq!(
            domain.resolve_url(&base_url, Some("eth_blockNumber"), None).url,
            "https://fallback.example.com/v2/rpc"
        );
    }

    #[test]
    fn resolve_url_with_path_override() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: None,
            path: Some("/api/v1/block".to_string()),
            url: "https://api.example.com/v2/block".to_string(),
        }]);
        let base_url = make_url("https://example.com");
        assert_eq!(
            domain.resolve_url(&base_url, None, Some("/api/v1/block")).url,
            "https://api.example.com/v2/block"
        );
    }

    #[test]
    fn resolve_url_preserves_headers() {
        let domain = make_domain_with_overrides(vec![Override {
            rpc_method: Some("eth_sendTransaction".to_string()),
            path: None,
            url: "https://tx-relay.example.com".to_string(),
        }]);
        let base_url = Url {
            url: "https://example.com/rpc".to_string(),
            headers: Some(std::collections::HashMap::from([("x-api-key".to_string(), "test123".to_string())])),
        };
        let resolved = domain.resolve_url(&base_url, Some("eth_sendTransaction"), None);
        assert_eq!(resolved.headers.as_ref().unwrap().get("x-api-key").unwrap(), "test123");
    }
}
