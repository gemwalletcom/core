use crate::models::account::PolkadotAccountBalance;
use num_bigint::BigUint;
use primitives::{AssetBalance, Balance, Chain};

pub fn map_coin_balance(balance: PolkadotAccountBalance) -> AssetBalance {
    let available = std::cmp::max(&balance.free - &balance.reserved, num_bigint::BigInt::from(0));

    AssetBalance::new_balance(
        Chain::Polkadot.as_asset_id(),
        Balance::with_reserved(
            BigUint::try_from(available).unwrap_or_default(),
            BigUint::try_from(balance.reserved).unwrap_or_default(),
        ),
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
        assert_eq!(result.balance.available, BigUint::from(900000000000_u64));
        assert_eq!(result.balance.reserved, BigUint::from(100000000000_u64));
    }
}
