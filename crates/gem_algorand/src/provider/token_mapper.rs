use crate::models::AssetDetails;
use primitives::{Asset, AssetId, AssetType, Chain};

pub fn is_valid_token_id(token_id: &str) -> bool {
    if token_id.len() > 4 && token_id.parse::<u64>().is_ok() {
        return true;
    }
    false
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
        assert!(is_valid_token_id("31566704"));
        assert!(!is_valid_token_id("abc"));
        assert!(!is_valid_token_id("12"));
    }
}
