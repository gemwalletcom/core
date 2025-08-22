use crate::rpc::model::AccountObject;
use primitives::{Asset, AssetId, AssetType, Chain};
use std::error::Error;

const XRP_TOKEN_DECIMALS: i32 = 15;

pub fn map_currency_hex_to_symbol(currency: &str) -> Result<String, Box<dyn Error + Sync + Send>> {
    let currency_bytes = hex::decode(currency.trim_end_matches('0')).map_err(|_| "Invalid currency hex")?;
    let symbol = String::from_utf8(currency_bytes.into_iter().filter(|&b| b != 0).collect()).unwrap_or_else(|_| currency.to_string());
    Ok(symbol)
}

pub fn map_token_data(object: &AccountObject, token_id: String, chain: Chain) -> Result<Asset, Box<dyn Error + Sync + Send>> {
    let symbol = map_currency_hex_to_symbol(&object.low_limit.currency)?;

    Ok(Asset {
        id: AssetId::from_token(chain, &token_id),
        chain,
        token_id: Some(token_id.clone()),
        name: symbol.clone(),
        symbol,
        decimals: XRP_TOKEN_DECIMALS,
        asset_type: AssetType::TOKEN,
    })
}

pub fn is_valid_token_address(token_id: &str) -> bool {
    token_id.len() == 34 && token_id.starts_with('r')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_token_address() {
        assert_eq!(is_valid_token_address("rN7n7otQDd6FczFgLdSqtcsAUxDkw6fzRH"), true);
        assert_eq!(is_valid_token_address("invalid"), false);
    }
}
