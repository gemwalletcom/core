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

#[derive(Debug, Clone)]
pub struct NodeResult {
    pub url: Url,
    pub block_number: u64,
    pub latency: u64,
}
