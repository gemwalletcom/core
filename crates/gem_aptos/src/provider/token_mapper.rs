use crate::models::{CoinInfo, Resource};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        APTOS_NATIVE_COIN,
        models::{CoinInfo, Resource},
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
}
