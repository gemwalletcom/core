use crate::models::account::StellarAccount;
use number_formatter::BigNumberFormatter;
use primitives::{AssetBalance, AssetId, Balance, Chain};
use std::error::Error;

const STELLAR_DECIMALS: u32 = 7;

pub fn map_native_balance(account: &StellarAccount, asset_id: AssetId, chain: Chain) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
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

    Ok(AssetBalance::new_balance(asset_id, Balance::with_reserved(available, reserved_str)))
}

pub fn map_token_balances(account: &StellarAccount, token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            // Parse issuer::symbol format only
            if let Some((issuer, symbol)) = token_id.split_once("::") {
                if let Some(balance) = account
                    .balances
                    .iter()
                    .find(|b| b.asset_issuer.as_deref() == Some(issuer) && b.asset_code.as_deref() == Some(symbol) && b.asset_type != "native")
                {
                    let amount = BigNumberFormatter::value_from_amount(&balance.balance, STELLAR_DECIMALS).unwrap_or("0".to_owned());
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(amount), true)
                } else {
                    AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance("0".to_owned()), false)
                }
            } else {
                // Invalid format - only support issuer::symbol
                AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance("0".to_owned()), false)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::account::StellarAccount;
    use serde_json;

    #[test]
    fn test_map_native_balance() {
        let account: StellarAccount = serde_json::from_str(include_str!("../../testdata/balance.json")).unwrap();
        let chain = Chain::Stellar;
        let asset_id = AssetId::from_chain(chain);

        let result = map_native_balance(&account, asset_id.clone(), chain).unwrap();

        assert_eq!(result.asset_id, asset_id);
        assert_eq!(result.balance.available, "299999077");
        assert_eq!(result.balance.reserved, "10000000");
    }
}
