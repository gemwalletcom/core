use crate::models::{balance::StakeBalance, token::SpotMetadataResponse};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_balance_coin(balance: String, chain: Chain) -> AssetBalance {
    AssetBalance::new(chain.as_asset_id(), balance.parse::<BigUint>().unwrap_or_default())
}

pub fn map_balance_token(balance: String, token_id: String, decimals: i32, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let available = BigNumberFormatter::value_from_amount_biguint(&balance, decimals as u32)?;
    let asset_id = AssetId::from(chain, Some(token_id));
    Ok(AssetBalance::new(asset_id, available))
}

pub fn map_balance_tokens(
    spot_balances: &crate::models::balance::Balances,
    spot_metadata: &SpotMetadataResponse,
    token_ids: &[String],
    chain: Chain,
) -> Vec<AssetBalance> {
    token_ids
        .iter()
        .filter_map(|token_id| {
            let parts = AssetId::decode_token_id(token_id);
            let symbol = parts.first()?;
            let token = spot_metadata.tokens.iter().find(|t| &t.name == symbol)?;
            let balance = spot_balances.balances.iter().find(|b| b.token == token.index as u32)?;
            map_balance_token(balance.total.clone(), token_id.clone(), token.wei_decimals, chain).ok()
        })
        .collect()
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
    use crate::models::{
        balance::{Balance, Balances, StakeBalance},
        token::{SpotMetadataResponse, SpotToken},
    };
    use primitives::Chain;

    #[test]
    fn test_map_balance_coin() {
        let balance = "1000000000000000000".to_string();
        let result = map_balance_coin(balance, Chain::SmartChain);

        assert_eq!(result.balance.available, BigUint::from(1000000000000000000_u64));
        assert_eq!(result.asset_id.chain, Chain::SmartChain);
    }

    #[test]
    fn test_map_balance_token() {
        let result = map_balance_token("56003537".to_string(), "USDC::0".to_string(), 8, Chain::HyperCore).unwrap();

        assert_eq!(result.balance.available, "5600353700000000".parse::<BigUint>().unwrap());
        assert_eq!(result.asset_id.chain, Chain::HyperCore);
        assert_eq!(result.asset_id.token_id, Some("USDC::0".to_string()));
    }

    #[test]
    fn test_map_balance_tokens() {
        let spot_balances = Balances {
            balances: vec![Balance {
                coin: "USDC".to_string(),
                token: 0,
                total: "56003537".to_string(),
            }],
        };

        let spot_metadata = SpotMetadataResponse {
            tokens: vec![SpotToken {
                name: "USDC".to_string(),
                wei_decimals: 8,
                index: 0,
                token_id: "0x6d1e7cde53ba9467b783cb7c530ce054".to_string(),
            }],
        };

        let token_ids_by_symbol = vec!["USDC".to_string()];
        let results = map_balance_tokens(&spot_balances, &spot_metadata, &token_ids_by_symbol, Chain::HyperCore);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].asset_id.chain, Chain::HyperCore);
        assert_eq!(results[0].balance.available, "5600353700000000".parse::<BigUint>().unwrap());

        let token_ids_full = vec!["USDC::0x6d1e7cde53ba9467b783cb7c530ce054::0".to_string()];
        let results_full = map_balance_tokens(&spot_balances, &spot_metadata, &token_ids_full, Chain::HyperCore);

        assert_eq!(results_full.len(), 1);
        assert_eq!(results_full[0].asset_id.chain, Chain::HyperCore);
        assert_eq!(results_full[0].balance.available, "5600353700000000".parse::<BigUint>().unwrap());
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
