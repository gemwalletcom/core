use primitives::{AssetBalance, BitcoinChain};
use crate::typeshare::account::BitcoinAccount;

pub fn map_balance_coin(account: &BitcoinAccount, chain: BitcoinChain) -> AssetBalance {
    AssetBalance::new(chain.get_chain().as_asset_id(), account.balance.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeshare::account::BitcoinAccount;
    use primitives::BitcoinChain;

    #[test]
    fn test_map_balance_coin() {
        let account = BitcoinAccount {
            balance: "100000000".to_string(),
        };
        let result = map_balance_coin(&account, BitcoinChain::Bitcoin);
        
        assert_eq!(result.balance.available, "100000000");
        assert_eq!(result.asset_id.chain, primitives::Chain::Bitcoin);
    }
}