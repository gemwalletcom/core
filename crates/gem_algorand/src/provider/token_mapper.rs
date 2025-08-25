use crate::models::rpc::AssetDetails;
use primitives::{Asset, AssetId, AssetType, Chain};

pub fn map_token_data(asset_info: &AssetDetails, token_id: String, chain: Chain) -> Asset {
    Asset {
        id: AssetId {
            chain,
            token_id: Some(token_id.clone()),
        },
        chain,
        token_id: Some(token_id),
        name: asset_info.params.name.clone(),
        symbol: asset_info.params.unit_name.clone(),
        decimals: asset_info.params.decimals as i32,
        asset_type: AssetType::ASA,
    }
}

pub fn is_valid_token_id(token_id: &str) -> bool {
    if token_id.len() > 4 {
        token_id.parse::<u64>().is_ok()
    } else {
        false
    }
}

pub fn map_asset(asset: AssetDetails) -> Asset {
    Asset::new(
        AssetId::from_token(Chain::Algorand, &asset.index.to_string()),
        asset.params.name,
        asset.params.unit_name,
        asset.params.decimals as i32,
        AssetType::TOKEN,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_token_id() {
        assert_eq!(is_valid_token_id("31566704"), true);
        assert_eq!(is_valid_token_id("abc"), false);
        assert_eq!(is_valid_token_id("12"), false);
    }
}
