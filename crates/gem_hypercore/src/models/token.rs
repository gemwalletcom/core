use std::sync::LazyLock;

use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};

pub const HYPERCORE_USDC_TOKEN_ID: &str = "USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0";
pub static HYPERCORE_USDC_ASSET_ID: LazyLock<AssetId> = LazyLock::new(|| AssetId::from(Chain::HyperCore, Some(HYPERCORE_USDC_TOKEN_ID.to_string())));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotToken {
    pub name: String,
    pub wei_decimals: i32,
    pub index: i32,
    pub token_id: String,
    pub sz_decimals: u32,
}

impl SpotToken {
    pub fn asset_id(&self, chain: Chain) -> AssetId {
        let token_id = AssetId::sub_token_id(&[self.name.clone(), self.token_id.clone(), self.index.to_string()]);
        AssetId::from(chain, Some(token_id))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotTokensResponse {
    pub tokens: Vec<SpotToken>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let token = SpotToken {
            name: "USDC".to_string(),
            wei_decimals: 8,
            index: 0,
            token_id: "0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
            sz_decimals: 2,
        };
        let asset_id = token.asset_id(Chain::HyperCore);

        assert_eq!(asset_id.chain, Chain::HyperCore);
        assert_eq!(asset_id.token_id, Some(HYPERCORE_USDC_TOKEN_ID.to_string()));
    }
}
