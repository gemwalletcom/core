use crate::models::balance::StakeBalance;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, Balance, Chain};
use std::error::Error;

pub fn map_balance_coin(balance: String, chain: Chain) -> AssetBalance {
    AssetBalance::new(chain.as_asset_id(), balance.parse::<BigUint>().unwrap_or_default())
}

pub fn map_balance_staking(balance: &StakeBalance, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available_biguint = BigNumberFormatter::value_from_amount_biguint(&balance.delegated.to_string(), 18).unwrap_or_default();
    let pending_biguint = BigNumberFormatter::value_from_amount_biguint(&balance.total_pending_withdrawal.to_string(), 18).unwrap_or_default();

    Ok(AssetBalance::new_balance(
        chain.as_asset_id(),
        Balance::stake_balance(available_biguint, pending_biguint, None),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::balance::StakeBalance;
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let balance = "1000000000000000000".to_string();
        let result = map_balance_coin(balance, Chain::SmartChain);

        assert_eq!(result.balance.available, BigUint::from(1000000000000000000_u64));
        assert_eq!(result.asset_id.chain, Chain::SmartChain);
    }

    #[test]
    fn test_map_balance_staking() {
        let stake_balance = StakeBalance {
            delegated: 1000000000000000000.0,
            undelegated: 0.0,
            total_pending_withdrawal: 100000000000000000.0,
        };
        let result = map_balance_staking(&stake_balance, Chain::SmartChain).unwrap();

        assert_eq!(result.balance.staked, "1000000000000000000000000000000000000".parse::<BigUint>().unwrap());
        assert_eq!(result.balance.pending, "100000000000000000000000000000000000".parse::<BigUint>().unwrap());
    }
}
