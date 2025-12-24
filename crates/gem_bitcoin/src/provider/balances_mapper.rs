use crate::models::account::BitcoinAccount;
use primitives::{AssetBalance, Balance, BitcoinChain};

pub fn map_balance_coin(account: &BitcoinAccount, chain: BitcoinChain) -> AssetBalance {
    let pending_unconfirmed = account.unconfirmed_balance.to_biguint().unwrap_or_default();
    let balance = Balance::with_pending_unconfirmed(account.balance.clone(), pending_unconfirmed);
    AssetBalance::new_balance(chain.get_chain().as_asset_id(), balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::{BigInt, BigUint};

    #[test]
    fn test_map_balance_coin_positive_unconfirmed() {
        let account = BitcoinAccount {
            balance: BigUint::from(20998955_u64),
            unconfirmed_balance: BigInt::from(5100000_i64),
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::from(20998955_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::from(5100000_u64));
    }

    #[test]
    fn test_map_balance_coin_negative_unconfirmed() {
        let account = BitcoinAccount {
            balance: BigUint::from(20000000_u64),
            unconfirmed_balance: BigInt::from(-10001045_i64),
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::from(20000000_u64));
        assert_eq!(result.balance.pending_unconfirmed, BigUint::ZERO);
    }
}
