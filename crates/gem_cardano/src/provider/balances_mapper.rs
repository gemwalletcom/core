use primitives::{AssetBalance, Chain};

pub fn map_balance_coin(balance: String, chain: Chain) -> AssetBalance {
    AssetBalance::new(chain.as_asset_id(), balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let balance = "1000000".to_string();
        let result = map_balance_coin(balance, Chain::Cardano);

        assert_eq!(result.balance.available, "1000000");
        assert_eq!(result.asset_id.chain, Chain::Cardano);
    }
}
