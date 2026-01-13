use crate::models::{CoinInfo, Metadata, Resource};
use primitives::{Asset, AssetId, AssetType, Chain};
use std::error::Error;

pub fn map_token_data(resource: &Resource<CoinInfo>, token_id: &str) -> Result<Asset, Box<dyn Error + Sync + Send>> {
    let coin_info = &resource.data;

    Ok(Asset {
        id: AssetId::from_token(Chain::Aptos, token_id),
        chain: Chain::Aptos,
        token_id: Some(token_id.to_string()),
        name: coin_info.name.clone(),
        symbol: coin_info.symbol.clone(),
        decimals: coin_info.decimals as i32,
        asset_type: AssetType::TOKEN,
    })
}

pub fn map_fungible_asset_metadata(resource: &Resource<Metadata>, token_id: &str) -> Result<Asset, Box<dyn Error + Sync + Send>> {
    let metadata = &resource.data;

    Ok(Asset {
        id: AssetId::from_token(Chain::Aptos, token_id),
        chain: Chain::Aptos,
        token_id: Some(token_id.to_string()),
        name: metadata.name.clone(),
        symbol: metadata.symbol.clone(),
        decimals: metadata.decimals as i32,
        asset_type: AssetType::TOKEN,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        APTOS_NATIVE_COIN,
        models::{CoinInfo, Metadata, Resource},
    };

    #[test]
    fn test_map_token_data() {
        let coin_info = CoinInfo {
            name: "Aptos Coin".to_string(),
            symbol: "APT".to_string(),
            decimals: 8,
        };

        let resource = Resource {
            type_field: "0x1::coin::CoinInfo<0x1::aptos_coin::AptosCoin>".to_string(),
            data: coin_info,
        };

        let result = map_token_data(&resource, APTOS_NATIVE_COIN).unwrap();

        assert_eq!(result.name, "Aptos Coin");
        assert_eq!(result.symbol, "APT");
        assert_eq!(result.decimals, 8);
        assert_eq!(result.asset_type, AssetType::TOKEN);
        assert_eq!(result.id.chain, Chain::Aptos);
    }

    #[test]
    fn test_map_fungible_asset_metadata() {
        let metadata = Metadata {
            decimals: 6,
            name: "Tether USD".to_string(),
            symbol: "USDt".to_string(),
        };

        let resource = Resource {
            type_field: "0x1::fungible_asset::Metadata".to_string(),
            data: metadata,
        };

        let token_id = "0x357b0b74bc833e95a115ad22604854d6b0fca151cecd94111770e5d6ffc9dc2b";
        let result = map_fungible_asset_metadata(&resource, token_id).unwrap();

        assert_eq!(result.symbol, "USDt");
        assert_eq!(result.decimals, 6);
        assert_eq!(result.id.token_id, Some(token_id.to_string()));
    }
}
