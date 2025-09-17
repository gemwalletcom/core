use crate::constants::STAKING_VALIDATORS_LIMIT;
use crate::rpc::client::EthereumClient;
use chrono::DateTime;
use gem_client::Client;
use num_bigint::{BigInt, BigUint};
use primitives::{AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};
use std::error::Error;
use std::str::FromStr;

#[cfg(feature = "rpc")]
impl<C: Client + Clone> EthereumClient<C> {
    pub async fn get_smartchain_validators(&self, _apy: f64) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>> {
        use alloy_primitives::hex::encode_prefixed;
        use gem_bsc::stake_hub::{decode_validators_return, encode_validators_call, HUB_READER_ADDRESS};

        let limit = self.get_max_elected_validators().await?;
        let call_data = encode_validators_call(0, limit);

        let call = (
            "eth_call".to_string(),
            serde_json::json!([{
            "to": HUB_READER_ADDRESS,
            "data": encode_prefixed(&call_data)
        }, "latest"]),
        );

        let result: String = self.call(call.0, call.1).await?;
        let result_data = hex::decode(result.trim_start_matches("0x"))?;
        let validators = decode_validators_return(&result_data)?;

        Ok(validators
            .into_iter()
            .map(|v| DelegationValidator {
                id: v.operator_address.clone(),
                chain: Chain::SmartChain,
                name: v.moniker,
                is_active: !v.jailed,
                commision: v.commission as f64 / 10000.0,
                apr: v.apy as f64 / 100.0,
            })
            .collect())
    }

    pub async fn get_smartchain_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>> {
        let validators = self.get_smartchain_validators(0.0).await?;
        let max_apr = validators
            .into_iter()
            .filter(|validator| validator.is_active)
            .filter_map(|validator| if validator.apr.is_finite() { Some(validator.apr) } else { None })
            .fold(None, |acc: Option<f64>, apr| match acc {
                Some(current) if current >= apr => Some(current),
                _ => Some(apr),
            });
        Ok(max_apr)
    }

    pub async fn get_smartchain_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>> {
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

        let results: Vec<String> = self.client.batch_call::<String>(calls).await?.extract();

        let delegations_data = hex::decode(results[0].trim_start_matches("0x"))?;
        let delegations = decode_delegations_return(&delegations_data)?;

        let undelegations_data = hex::decode(results[1].trim_start_matches("0x"))?;
        let undelegations = decode_undelegations_return(&undelegations_data)?;

        let mut result = Vec::new();

        let asset_id = AssetId {
            chain: self.get_chain(),
            token_id: None,
        };

        for delegation in delegations {
            if let Ok(balance_uint) = BigUint::from_str(&delegation.amount) {
                let balance = BigInt::from(balance_uint);
                let shares = if let Ok(shares_uint) = BigUint::from_str(&delegation.shares) {
                    BigInt::from(shares_uint)
                } else {
                    BigInt::from(0u32)
                };

                result.push(DelegationBase {
                    asset_id: asset_id.clone(),
                    delegation_id: delegation.delegator_address.clone(),
                    validator_id: delegation.validator_address,
                    balance,
                    shares,
                    rewards: BigInt::from(0u32),
                    completion_date: None,
                    state: DelegationState::Active,
                });
            }
        }

        for undelegation in undelegations {
            if let Ok(balance_uint) = BigUint::from_str(&undelegation.amount) {
                let balance = BigInt::from(balance_uint);
                let shares = if let Ok(shares_uint) = BigUint::from_str(&undelegation.shares) {
                    BigInt::from(shares_uint)
                } else {
                    BigInt::from(0u32)
                };

                let completion_date = if let Ok(unlock_time) = undelegation.unlock_time.parse::<i64>() {
                    Some(DateTime::from_timestamp(unlock_time, 0).unwrap_or_default())
                } else {
                    None
                };

                result.push(DelegationBase {
                    asset_id: asset_id.clone(),
                    delegation_id: undelegation.delegator_address.clone(),
                    validator_id: undelegation.validator_address,
                    balance,
                    shares,
                    rewards: BigInt::from(0u32),
                    completion_date,
                    state: DelegationState::Undelegating,
                });
            }
        }

        Ok(result)
    }

    async fn get_max_elected_validators(&self) -> Result<u16, Box<dyn Error + Sync + Send>> {
        use gem_bsc::stake_hub::STAKE_HUB_ADDRESS;

        let call = (
            "eth_call".to_string(),
            serde_json::json!([{
                "to": STAKE_HUB_ADDRESS,
                "data": "0xc473318f"
            }, "latest"]),
        );

        let result: String = self.call(call.0, call.1).await?;
        let result_data = hex::decode(result.trim_start_matches("0x"))?;

        if result_data.len() >= 32 {
            let value = u32::from_be_bytes([result_data[28], result_data[29], result_data[30], result_data[31]]) as u16;
            Ok(value)
        } else {
            Err("Invalid response format for maxElectedValidators".into())
        }
    }
}
