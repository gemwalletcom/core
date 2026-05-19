use crate::{AssetId, Chain, swap::QuoteAsset};

impl QuoteAsset {
    pub fn mock() -> Self {
        Self::mock_with_asset_id(AssetId::from_chain(Chain::Ethereum), "ETH", 18)
    }

    pub fn mock_with_asset_id(id: AssetId, symbol: &str, decimals: u32) -> Self {
        Self {
            id: id.to_string(),
            symbol: symbol.to_string(),
            decimals,
        }
    }
}
