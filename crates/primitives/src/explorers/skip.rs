use crate::block_explorer::BlockExplorer;
use crate::chain::Chain;

pub struct SkipExplorer {
    chain_id: String,
}

impl SkipExplorer {
    pub fn boxed(chain: Chain) -> Box<dyn BlockExplorer> {
        Box::new(Self {
            chain_id: chain.network_id().to_string(),
        })
    }
}

impl BlockExplorer for SkipExplorer {
    fn name(&self) -> String {
        "Skip Explorer".into()
    }

    fn get_tx_url(&self, hash: &str) -> String {
        format!("https://explorer.skip.build/?tx_hash={hash}&chain_id={}", self.chain_id)
    }

    fn get_address_url(&self, _address: &str) -> String {
        String::new()
    }

    fn get_token_url(&self, _token_id: &str) -> Option<String> {
        None
    }

    fn get_validator_url(&self, _address: &str) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_explorer_url() {
        let explorer = SkipExplorer::boxed(Chain::Osmosis);
        assert_eq!(explorer.name(), "Skip Explorer");
        assert_eq!(
            explorer.get_tx_url("1FE2FF8C64062136544C35451E5AE292229A156E174C0EFF5B67970E629A8B1C"),
            "https://explorer.skip.build/?tx_hash=1FE2FF8C64062136544C35451E5AE292229A156E174C0EFF5B67970E629A8B1C&chain_id=osmosis-1"
        );
    }
}
