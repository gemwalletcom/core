use crate::models::{DelegationPoolStake, Resource, ResourceData};
use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Balance, Chain};

pub fn map_native_balance(balance: &BigUint, chain: Chain) -> AssetBalance {
    let asset_id = AssetId::from_chain(chain);
    AssetBalance::new(asset_id, balance.clone())
}

pub fn map_balance_tokens(balances: Vec<(String, u64)>, chain: Chain) -> Vec<AssetBalance> {
    balances
        .into_iter()
        .map(|(token_id, balance)| {
            let asset_id = AssetId::from_token(chain, &token_id);
            AssetBalance::new(asset_id, BigUint::from(balance))
        })
        .collect()
}

pub fn map_token_balances(resources: &[Resource<ResourceData>], token_ids: Vec<String>, chain: Chain) -> Vec<AssetBalance> {
    token_ids
        .into_iter()
        .map(|token_id| {
            let coin_store_type = format!("0x1::coin::CoinStore<{}>", token_id);
            let balance = resources
                .iter()
                .find(|r| r.type_field == coin_store_type)
                .and_then(|resource| resource.data.coin.as_ref())
                .map(|coin_data| coin_data.value.clone())
                .unwrap_or_else(|| BigUint::from(0u32));

            AssetBalance::new_with_active(AssetId::from_token(chain, &token_id), Balance::coin_balance(balance), true)
        })
        .collect()
}

pub fn map_balance_staking(stake: DelegationPoolStake, chain: Chain) -> AssetBalance {
    let staked = stake.active;
    let pending = &stake.pending_inactive + &stake.inactive;
    let balance = Balance::stake_balance(staked, pending, None);

    AssetBalance::new_balance(AssetId::from_chain(chain), balance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        APTOS_NATIVE_COIN,
        models::{CoinData, Resource, ResourceData},
    };

    #[test]
    fn test_map_native_balance() {
        let balance = BigUint::from(1000000_u64);
        let result = map_native_balance(&balance, Chain::Aptos);

        assert_eq!(result.balance.available, BigUint::from(1000000_u64));
        assert_eq!(result.asset_id.chain, Chain::Aptos);
        assert_eq!(result.asset_id.token_id, None);
    }

    #[test]
    fn test_map_balance_tokens() {
        let balances = vec![
            (
                "0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT".to_string(),
                25379808,
            ),
            (APTOS_NATIVE_COIN.to_string(), 1000000),
        ];

        let result = map_balance_tokens(balances, Chain::Aptos);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].balance.available, BigUint::from(25379808_u64));
        assert_eq!(result[0].asset_id.chain, Chain::Aptos);
        assert_eq!(
            result[0].asset_id.token_id,
            Some("0x159df6b7689437016108a019fd5bef736bac692b6d4a1f10c941f6fbb9a74ca6::oft::CakeOFT".to_string())
        );

        assert_eq!(result[1].balance.available, BigUint::from(1000000_u64));
        assert_eq!(result[1].asset_id.chain, Chain::Aptos);
        assert_eq!(result[1].asset_id.token_id, Some(APTOS_NATIVE_COIN.to_string()));
    }

    #[test]
    fn test_map_token_balances() {
        let coin_data = CoinData {
            value: BigUint::from(1000000_u64),
        };

        let resource = Resource {
            type_field: "0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>".to_string(),
            data: ResourceData { coin: Some(coin_data) },
        };

        let resources = vec![resource];
        let token_ids = vec![APTOS_NATIVE_COIN.to_string()];

        let result = map_token_balances(&resources, token_ids, Chain::Aptos);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].balance.available, BigUint::from(1000000_u64));
        assert!(result[0].is_active);
    }
}
