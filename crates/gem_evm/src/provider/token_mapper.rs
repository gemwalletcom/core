use crate::{
    contracts::erc20::{decode_abi_string, decode_abi_uint8},
    ethereum_address_checksum,
};
use primitives::{Asset, AssetId, Chain};

pub fn map_token_data(
    chain: Chain,
    token_id: String,
    name_hex: String,
    symbol_hex: String,
    decimals_hex: String,
) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
    let name = decode_abi_string(name_hex.trim_start_matches("0x"))?;
    let symbol = decode_abi_string(symbol_hex.trim_start_matches("0x"))?;
    let decimals = decode_abi_uint8(decimals_hex.trim_start_matches("0x"))?;
    let token_id = ethereum_address_checksum(&token_id)?;

    if name.is_empty() {
        return Err("Invalid token metadata: name is empty".into());
    }
    if symbol.is_empty() {
        return Err("Invalid token metadata: symbol is empty".into());
    }

    let asset_id = AssetId {
        chain,
        token_id: Some(token_id.clone()),
    };

    Ok(Asset::new(
        asset_id.clone(),
        name,
        symbol,
        decimals.into(),
        asset_id.chain.default_asset_type().unwrap(),
    ))
}

pub fn map_is_token_address(token_id: &str) -> bool {
    token_id.starts_with("0x") && token_id.len() == 42
}

#[cfg(test)]
mod tests {
    use primitives::AssetType;

    use super::*;

    #[test]
    fn test_map_is_token_address() {
        assert!(map_is_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
        assert!(!map_is_token_address("0x1234"));
        assert!(!map_is_token_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48123"));
        assert!(!map_is_token_address("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
        assert!(!map_is_token_address(""));
        assert!(!map_is_token_address("0x"));
    }

    #[test]
    fn test_map_token_data() {
        let token_id = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string();
        let chain = Chain::Ethereum;
        let name_hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000855534420436f696e000000000000000000000000000000000000000000000000".to_string();
        let symbol_hex = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000045553444300000000000000000000000000000000000000000000000000000000".to_string();
        let decimals_hex = "0x0000000000000000000000000000000000000000000000000000000000000006".to_string();

        let result = map_token_data(chain, token_id.clone(), name_hex, symbol_hex, decimals_hex).unwrap();

        assert_eq!(result.name, "USD Coin");
        assert_eq!(result.symbol, "USDC");
        assert_eq!(result.decimals, 6);
        assert_eq!(result.id.chain, Chain::Ethereum);
        assert_eq!(result.chain, Chain::Ethereum);
        assert_eq!(result.token_id, Some("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()));
        assert_eq!(result.asset_type, AssetType::ERC20);
    }

    #[test]
    fn test_map_token_data_invalid_metadata() {
        let token_id = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string();
        let name_hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000855534420436f696e000000000000000000000000000000000000000000000000".to_string();
        let symbol_hex = "0x000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000045553444300000000000000000000000000000000000000000000000000000000".to_string();
        let decimals_hex = "0x0000000000000000000000000000000000000000000000000000000000000006".to_string();

        assert!(map_token_data(Chain::Ethereum, token_id.clone(), "".to_string(), symbol_hex, decimals_hex.clone()).is_err());
        assert!(map_token_data(Chain::Ethereum, token_id.clone(), name_hex, "".to_string(), decimals_hex.clone()).is_err());
        assert!(map_token_data(Chain::Ethereum, token_id, "".to_string(), "".to_string(), decimals_hex).is_err());
    }
}
