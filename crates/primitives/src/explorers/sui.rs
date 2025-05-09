use crate::block_explorer::{BlockExplorer, Metadata};

pub struct SuiScan {
    pub meta: Metadata,
}

impl SuiScan {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "SuiScan",
                base_url: "https://suiscan.xyz/mainnet",
            },
        })
    }
}

impl BlockExplorer for SuiScan {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/tx/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        format!("{}/coin/{}", self.meta.base_url, token).into()
    }
    fn get_validator_url(&self, validator: &str) -> Option<String> {
        format!("{}/validator/{}", self.meta.base_url, validator).into()
    }
}

pub struct SuiVision {
    pub meta: Metadata,
}

impl SuiVision {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            meta: Metadata {
                name: "SuiVision",
                base_url: "https://suivision.xyz",
            },
        })
    }
}
impl BlockExplorer for SuiVision {
    fn name(&self) -> String {
        self.meta.name.into()
    }
    fn get_tx_url(&self, hash: &str) -> String {
        format!("{}/txblock/{}", self.meta.base_url, hash)
    }
    fn get_address_url(&self, address: &str) -> String {
        format!("{}/account/{}", self.meta.base_url, address)
    }
    fn get_token_url(&self, token: &str) -> Option<String> {
        format!("{}/coin/{}", self.meta.base_url, token).into()
    }

    fn get_validator_url(&self, validator: &str) -> Option<String> {
        format!("{}/validator/{}", self.meta.base_url, validator).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_url() {
        let explorer = SuiScan::new();
        let token = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
        assert_eq!(
            explorer.get_token_url(token).unwrap(),
            "https://suiscan.xyz/mainnet/coin/0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC"
        );
    }
}
