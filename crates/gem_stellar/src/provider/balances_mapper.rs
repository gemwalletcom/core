use crate::constants::STELLAR_DECIMALS;
use crate::models::account::Account;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

pub fn map_native_balance(account: &Account) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
    let chain = Chain::Stellar;
    let reserved_amount = chain.account_activation_fee().unwrap_or(0) as u64;
    let native_balance = account
        .balances
        .iter()
        .find(|b| b.asset_type == "native")
        .map(|b| b.balance.clone())
        .unwrap_or_default();

    let balance_stroops_str = BigNumberFormatter::value_from_amount(&native_balance, STELLAR_DECIMALS)?;
    let balance_decimal = BigNumberFormatter::big_decimal_value(&balance_stroops_str, 0).unwrap_or_default();
    let reserved_decimal = BigNumberFormatter::big_decimal_value(&reserved_amount.to_string(), 0).unwrap_or_default();
    let available_decimal = balance_decimal - reserved_decimal;
    let available = available_decimal.to_string();
    let reserved_str = reserved_amount.to_string();

    let available_biguint = available.parse::<BigUint>().unwrap_or_default();
    let reserved_biguint = reserved_str.parse::<BigUint>().unwrap_or_default();

    Ok(AssetBalance::new_balance(
        chain.as_asset_id(),
        Balance::with_reserved(available_biguint, reserved_biguint),
    ))
}

pub fn map_token_balances(account: &Account, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            if let Some((issuer, symbol)) = token_id.split_once("::") {
                if let Some(balance) = account
                    .balances
                    .iter()
                    .find(|b| b.asset_issuer.as_deref() == Some(issuer) && b.asset_code.as_deref() == Some(symbol) && b.asset_type != "native")
                {
                    let amount = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS).unwrap_or_default();
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(amount), true)
                } else {
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(BigUint::from(0u32)), false)
                }
            } else {
                // Invalid format - only support issuer::symbol
                AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(BigUint::from(0u32)), false)
            }
        })
        .collect()
}

pub fn map_all_balances(chain: Chain, account: Account) -> Vec<AssetBalance> {
    let mut balances = Vec::new();

    for balance in account.balances {
        match balance.asset_type.as_str() {
            "native" => {
                // Native XLM balance
                if let Ok(value) = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS) {
                    let balance_obj = Balance::coin_balance(value);
                    balances.push(AssetBalance::new_with_active(chain.as_asset_id(), balance_obj, true));
                }
            }
            "credit_alphanum4" | "credit_alphanum12" => {
                // Token balances
                if let (Some(asset_issuer), Some(asset_code)) = (&balance.asset_issuer, &balance.asset_code) {
                    let token_id = format!("{}-{}", asset_code, asset_issuer);
                    let asset_id = AssetId::from_token(chain, &token_id);
                    if let Ok(value) = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS) {
                        let balance_obj = Balance::coin_balance(value);
                        balances.push(AssetBalance::new_with_active(asset_id, balance_obj, true));
                    }
                }
            }
            _ => {
                // Ignore other asset types
            }
        }
    }

    balances
}

pub fn map_token_balances_by_ids(chain: Chain, account: &Account, token_ids: &[String]) -> Vec<AssetBalance> {
    let mut result = Vec::new();
    for token_id in token_ids {
        if let Some(balance) = account.balances.iter().find(|b| {
            if let (Some(asset_issuer), Some(asset_code)) = (&b.asset_issuer, &b.asset_code) {
                let balance_token_id = format!("{}-{}", asset_code, asset_issuer);
                balance_token_id == *token_id && b.asset_type != "native"
            } else {
                false
            }
        }) {
            if let Ok(amount) = BigNumberFormatter::value_from_amount_biguint(&balance.balance, STELLAR_DECIMALS) {
                let asset_id = AssetId::from_token(chain, token_id);
                let balance_obj = Balance::coin_balance(amount);
                result.push(AssetBalance::new_with_active(asset_id, balance_obj, true));
            }
        } else {
            let asset_id = AssetId::from_token(chain, token_id);
            let balance_obj = Balance::coin_balance(BigUint::from(0u32));
            result.push(AssetBalance::new_with_active(asset_id, balance_obj, false));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::Account;
    use serde_json;

    #[test]
    fn test_map_native_balance() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/balance.json")).unwrap();
        let chain = Chain::Stellar;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(299999077_u64));
        assert_eq!(result.balance.reserved, BigUint::from(10000000_u64));
    }

    #[test]
    fn test_map_native_balance_with_minimal_balance() {
        let account: Account = serde_json::from_str(include_str!("../../testdata/balance_coin.json")).unwrap();
        let chain = Chain::Stellar;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, BigUint::from(0u32));
        assert_eq!(result.balance.reserved, BigUint::from(10000000_u64));
    }
}
