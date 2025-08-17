use crate::models::balance::HypercoreStakeBalance;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, Balance, Chain};
use std::error::Error;

pub fn map_balance_coin(balance: String, chain: Chain) -> AssetBalance {
    AssetBalance::new(chain.as_asset_id(), balance)
}

pub fn map_balance_staking(balance: &HypercoreStakeBalance, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available = BigNumberFormatter::value_from_amount(&balance.delegated, 18)?;
    let pending = BigNumberFormatter::value_from_amount(&balance.total_pending_withdrawal, 18)?;

    Ok(AssetBalance::new_balance(chain.as_asset_id(), Balance::stake_balance(available, pending, None)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::HypercoreStakeBalance;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let balance = "1000000000000000000".to_string();
        let result = map_balance_coin(balance, Chain::SmartChain);

        assert_eq!(result.balance.available, "1000000000000000000");
        assert_eq!(result.asset_id.chain, Chain::SmartChain);
    }

    #[test]
    fn test_map_balance_staking() {
        let stake_balance = HypercoreStakeBalance {
            delegated: "1000000000000000000".to_string(),
            undelegated: "0".to_string(),
            total_pending_withdrawal: "100000000000000000".to_string(),
        };
        let result = map_balance_staking(&stake_balance, Chain::SmartChain).unwrap();

        assert_eq!(result.balance.staked, "1000000000000000000000000000000000000");
        assert_eq!(result.balance.pending, "100000000000000000000000000000000000");
    }
}
