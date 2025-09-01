use crate::models::SuiCoinMetadata;
use primitives::{Asset, AssetId, AssetType, Chain};

pub fn map_is_token_address(token_id: &str) -> bool {
    token_id.starts_with("0x") && token_id.len() >= 66 && token_id.len() <= 100
}

pub fn map_token_data(chain: Chain, token_id: &str, metadata: SuiCoinMetadata) -> Asset {
    Asset::new(
        AssetId::from_token(chain, token_id),
        metadata.name,
        metadata.symbol,
        metadata.decimals,
        AssetType::TOKEN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_is_token_address() {
        assert!(map_is_token_address(
            "0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN"
        ));
        assert!(!map_is_token_address("invalid"));
        assert!(!map_is_token_address("0x123"));
    }

    #[test]
    fn test_map_token_data() {
        let metadata = SuiCoinMetadata {
            name: "USD Coin".to_string(),
            symbol: "USDC".to_string(),
            decimals: 6,
        };
        let token_id = "0x5d4b302506645c37ff133b98c4b50a5ae14841659738d6d733d59d0d217a93bf::coin::COIN";

        let asset = map_token_data(Chain::Sui, token_id, metadata);

        assert_eq!(asset.name, "USD Coin");
        assert_eq!(asset.symbol, "USDC");
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.chain, Chain::Sui);
        assert_eq!(asset.asset_type, AssetType::TOKEN);
    }
}
