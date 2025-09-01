use primitives::{AssetBalance, Balance, Chain};
use serde_serializers::biguint_from_hex_str;
use std::error::Error;

pub fn map_balance_coin(balance_hex: String, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Send + Sync>> {
    Ok(AssetBalance::new_balance(
        chain.as_asset_id(),
        Balance::coin_balance(biguint_from_hex_str(&balance_hex)?),
    ))
}

pub fn map_balance_tokens(_balance_data: Vec<String>, _token_ids: Vec<String>) -> Vec<AssetBalance> {
    unimplemented!("map_balance_tokens")
}

pub fn map_balance_staking(_staking_data: String) -> Option<AssetBalance> {
    unimplemented!("map_balance_staking")
}

pub fn map_assets_balances(_balance_data: Vec<String>) -> Vec<AssetBalance> {
    unimplemented!("map_assets_balances")
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let result = map_balance_coin("0x1c6bf52634000".to_string(), Chain::Ethereum).unwrap();
        assert_eq!(result.asset_id.chain, Chain::Ethereum);
        assert_eq!(result.balance.available, BigUint::from(500000000000000_u64));
    }
}
