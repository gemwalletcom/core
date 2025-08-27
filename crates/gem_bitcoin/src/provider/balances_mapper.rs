use crate::models::account::BitcoinAccount;
use primitives::{AssetBalance, BitcoinChain};

pub fn map_balance_coin(account: &BitcoinAccount, chain: BitcoinChain) -> AssetBalance {
    AssetBalance::new(chain.get_chain().as_asset_id(), account.balance.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::BitcoinAccount;
    use num_bigint::BigUint;
    use primitives::BitcoinChain;

    #[test]
    fn test_map_balance_coin() {
        let account = BitcoinAccount {
            balance: BigUint::from(100000000_u64),
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);

        assert_eq!(result.balance.available, BigUint::from(100000000_u64));
        assert_eq!(result.asset_id.chain, primitives::Chain::Bitcoin);
    }
}
