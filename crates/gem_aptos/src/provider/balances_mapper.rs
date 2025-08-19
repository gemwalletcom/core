use crate::models::{Resource, ResourceData};

use primitives::{AssetBalance, Balance, AssetId, Chain};

pub fn map_native_balance(
    balance: &str,
    chain: Chain,
) -> AssetBalance {
    let asset_id = AssetId::from_chain(chain);
    AssetBalance::new(asset_id, balance.to_string())
}

pub fn map_token_balances(
    resources: &[Resource<ResourceData>],
    token_ids: Vec<String>,
    chain: Chain,
) -> Vec<AssetBalance> {
    token_ids.into_iter().map(|token_id| {
        let coin_store_type = format!("0x1::coin::CoinStore<{}>", token_id);
        let balance = resources
            .iter()
            .find(|r| r.type_field == coin_store_type)
            .and_then(|resource| resource.data.coin.as_ref())
            .map(|coin_data| coin_data.value.clone())
            .unwrap_or_else(|| "0".to_string());

        AssetBalance::new_with_active(
            AssetId::from_token(chain, &token_id),
            Balance::coin_balance(balance),
            true
        )
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::{CoinData, Resource, ResourceData}, APTOS_NATIVE_COIN};

    #[test]
    fn test_map_native_balance() {
        let balance = "1000000";
        let result = map_native_balance(balance, Chain::Aptos);
        
        assert_eq!(result.balance.available, "1000000");
        assert_eq!(result.asset_id.chain, Chain::Aptos);
        assert_eq!(result.asset_id.token_id, None);
    }

    #[test]
    fn test_map_token_balances() {
        let coin_data = CoinData {
            value: "1000000".to_string(),
        };
        
        let resource = Resource {
            type_field: "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>".to_string(),
            data: ResourceData {
                coin: Some(coin_data),
            },
        };
        
        let resources = vec![resource];
        let token_ids = vec![APTOS_NATIVE_COIN.to_string()];
        
        let result = map_token_balances(&resources, token_ids, Chain::Aptos);
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].balance.available, "1000000");
        assert_eq!(result[0].is_active, Some(true));
    }
}