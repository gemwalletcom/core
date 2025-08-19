use primitives::{AssetBalance, Balance, Chain};
use crate::models::account::PolkadotAccountBalance;

pub fn map_coin_balance(balance: PolkadotAccountBalance) -> AssetBalance {
    let free = balance.free.parse::<num_bigint::BigInt>().unwrap_or_default();
    let reserved = balance.reserved.parse::<num_bigint::BigInt>().unwrap_or_default();
    let available = std::cmp::max(free - &reserved, num_bigint::BigInt::from(0));
    
    AssetBalance::new_balance(
        Chain::Polkadot.as_asset_id(),
        Balance::with_reserved(available.to_string(), reserved.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_coin_balance() {
        let balance = PolkadotAccountBalance {
            free: "1000000000000".to_string(),
            reserved: "100000000000".to_string(),
            nonce: "1".to_string(),
        };

        let result = map_coin_balance(balance);

        assert_eq!(result.asset_id, Chain::Polkadot.as_asset_id());
        assert_eq!(result.balance.available, "900000000000");
        assert_eq!(result.balance.reserved, "100000000000");
    }
}