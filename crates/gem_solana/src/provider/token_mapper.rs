use crate::{
    metaplex::metadata::Metadata,
    models::{Extension, TokenInfo},
};
use primitives::{Asset, AssetId, AssetType, Chain};

pub fn map_token_data_metaplex(
    chain: Chain,
    token_id: String,
    token_info: &TokenInfo,
    meta: &Metadata,
) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
    let name = meta.data.name.trim_matches(char::from(0)).to_string();
    let symbol = meta.data.symbol.trim_matches(char::from(0)).to_string();
    let decimals = token_info.decimals;

    Ok(Asset::new(AssetId::from_token(chain, &token_id), name, symbol, decimals, AssetType::TOKEN))
}

pub fn map_token_data_spl_token_2022(chain: Chain, token_id: String, token_info: &TokenInfo) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
    let token_metadata = token_info
        .extensions
        .as_ref()
        .and_then(|extensions| {
            extensions.iter().find_map(|ext| {
                if let Extension::TokenMetadata(token_metadata) = ext {
                    Some(token_metadata.state.clone())
                } else {
                    None
                }
            })
        })
        .ok_or("no token metadata found")?;
    Ok(Asset::new(
        AssetId::from_token(chain, &token_id),
        token_metadata.name,
        token_metadata.symbol,
        token_info.decimals,
        AssetType::TOKEN,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ResultTokenInfo;
    use primitives::{AssetType, JsonRpcResult};

    #[test]
    fn test_map_token_spl_token_2022() {
        let file = include_str!("../../testdata/pyusd_mint.json");
        let result = serde_json::from_str::<JsonRpcResult<ResultTokenInfo>>(file)
            .unwrap()
            .result
            .value
            .data
            .parsed
            .info;

        let token_data = map_token_data_spl_token_2022(Chain::Solana, "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo".to_string(), &result).unwrap();

        assert_eq!(token_data.name, "PayPal USD");
        assert_eq!(token_data.symbol, "PYUSD");
        assert_eq!(token_data.decimals, 6);
        assert_eq!(token_data.asset_type, AssetType::TOKEN);
    }
}
