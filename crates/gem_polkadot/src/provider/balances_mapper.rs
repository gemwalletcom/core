use primitives::{AssetBalance, Balance, Chain};
use crate::models::account::PolkadotAccountBalance;

pub fn map_coin_balance(balance: PolkadotAccountBalance) -> AssetBalance {
    let available = std::cmp::max(&balance.free - &balance.reserved, num_bigint::BigInt::from(0));
    
    AssetBalance::new_balance(
        Chain::Polkadot.as_asset_id(),
        Balance::with_reserved(available.to_string(), balance.reserved.to_string()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_coin_balance() {
        let balance = PolkadotAccountBalance {
            free: num_bigint::BigInt::from(1000000000000_u64),
            reserved: num_bigint::BigInt::from(100000000000_u64),
            nonce: 1,
        };

        let result = map_coin_balance(balance);

        assert_eq!(result.asset_id, Chain::Polkadot.as_asset_id());
        assert_eq!(result.balance.available, "900000000000");
        assert_eq!(result.balance.reserved, "100000000000");
    }
}