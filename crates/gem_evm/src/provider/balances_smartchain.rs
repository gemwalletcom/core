use crate::constants::STAKING_VALIDATORS_LIMIT;
use crate::rpc::client::EthereumClient;
use gem_client::Client;
use gem_jsonrpc::types::JsonRpcResult;
use num_bigint::BigUint;
use primitives::{AssetBalance, Balance};
use std::error::Error;
use std::str::FromStr;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_smartchain_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>> {
        use alloy_primitives::hex::encode_prefixed;
        use gem_bsc::stake_hub::{
            decode_delegations_return, decode_undelegations_return, encode_delegations_call, encode_undelegations_call, HUB_READER_ADDRESS,
        };

        let delegations_call_data = encode_delegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?;
        let undelegations_call_data = encode_undelegations_call(address, 0, STAKING_VALIDATORS_LIMIT)?;

        let calls = vec![
            (
                "eth_call".to_string(),
                serde_json::json!([{
                    "to": HUB_READER_ADDRESS,
                    "data": encode_prefixed(&delegations_call_data)
                }, "latest"]),
            ),
            (
                "eth_call".to_string(),
                serde_json::json!([{
                    "to": HUB_READER_ADDRESS,
                    "data": encode_prefixed(&undelegations_call_data)
                }, "latest"]),
            ),
        ];

        let results: Vec<String> = self
            .batch_call::<String>(calls)
            .await?
            .into_iter()
            .map(|result| match result {
                JsonRpcResult::Value(value) => Ok(value.result),
                JsonRpcResult::Error(error) => Err(error.error),
            })
            .collect::<Result<Vec<_>, _>>()?;

        let delegations_data = hex::decode(results[0].trim_start_matches("0x"))?;
        let delegations = decode_delegations_return(&delegations_data)?;

        let undelegations_data = hex::decode(results[1].trim_start_matches("0x"))?;
        let undelegations = decode_undelegations_return(&undelegations_data)?;

        let staked = delegations
            .iter()
            .filter_map(|d| BigUint::from_str(&d.amount).ok())
            .fold(BigUint::from(0u32), |acc, amount| acc + amount);

        let pending = undelegations
            .iter()
            .filter_map(|u| BigUint::from_str(&u.amount).ok())
            .fold(BigUint::from(0u32), |acc, amount| acc + amount);

        Ok(Some(AssetBalance {
            asset_id: self.get_chain().as_asset_id(),
            balance: Balance::stake_balance(staked, pending, None),
            is_active: Some(true),
        }))
    }
}
