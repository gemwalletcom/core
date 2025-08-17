use primitives::{Asset, AssetId, AssetType, Chain};
use crate::typeshare::common::StellarAsset;

const STELLAR_TOKEN_DECIMALS: i32 = 7;

pub fn map_token_data(asset: &StellarAsset, token_id: String, chain: Chain) -> Asset {
    Asset {
        id: AssetId::from(chain, Some(token_id.clone())),
        chain,
        token_id: Some(token_id),
        name: asset.asset_code.clone(),
        symbol: asset.asset_code.clone(),
        decimals: STELLAR_TOKEN_DECIMALS,
        asset_type: AssetType::TOKEN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_token_data() {
        let stellar_asset = StellarAsset {
            asset_code: "USDC".to_string(),
            asset_issuer: "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string(),
            contract_id: None,
        };
        let token_id = "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN::USDC".to_string();
        let chain = Chain::Stellar;
        
        let result = map_token_data(&stellar_asset, token_id, chain);
        assert_eq!(result.symbol, "USDC");
        assert_eq!(result.chain, Chain::Stellar);
    }
}