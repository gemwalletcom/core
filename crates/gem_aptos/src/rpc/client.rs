use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_client::{CONTENT_TYPE, Client, ClientExt, ContentType, build_path_with_query};
use num_bigint::BigUint;
use primitives::chain::Chain;
use primitives::{StakeType, TransactionInputType, TransactionLoadInput};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::models::{
    Account, Block, DelegationPoolStake, GasFee, Ledger, ReconfigurationState, Resource, ResourceData, SimulateTransactionQuery, StakingConfig, Transaction, TransactionPayload,
    TransactionResponse, TransactionSignature, TransactionSimulation, ValidatorSet,
};
use crate::provider::payload_builder::{
    build_stake_transaction_payload, build_token_transfer_transaction_payload, build_transfer_transaction_payload, build_unstake_transaction_payload,
    build_withdraw_transaction_payload,
};
use crate::{DEFAULT_MAX_GAS_AMOUNT, DEFAULT_SWAP_MAX_GAS_AMOUNT};

#[derive(Debug)]
pub struct AptosClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> AptosClient<C> {
    pub fn new(client: C) -> Self {
        Self { client, chain: Chain::Aptos }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_ledger(&self) -> Result<Ledger, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/").await?)
    }

    pub async fn get_block_transactions(&self, block_number: u64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("/v1/blocks/by_height/{}?with_transactions=true", block_number);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let url = format!("/v1/accounts/{}/transactions", address);
        Ok(self.client.get(&url).await?)
    }

    pub async fn get_account_resource<T: Serialize + DeserializeOwned + Send>(&self, address: String, resource: &str) -> Result<Resource<T>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}/resource/{}", address, resource)).await?)
    }

    pub async fn get_account_balance(&self, address: &str, asset_type: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}/balance/{}", address, asset_type)).await?)
    }

    pub async fn get_account_resources(&self, address: &str) -> Result<Vec<Resource<ResourceData>>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}/resources", address)).await?)
    }

    pub async fn get_account(&self, address: &str) -> Result<Account, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}", address)).await?)
    }

    pub async fn submit_transaction(&self, bcs_bytes: Vec<u8>) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        let headers = HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationAptosBcs.as_str().to_string())]);
        let response = self
            .client
            .post_with_headers::<Vec<u8>, TransactionResponse>("/v1/transactions", &bcs_bytes, headers)
            .await?;

        if let Some(message) = &response.message {
            return Err(Box::new(std::io::Error::other(message.clone())));
        }

        Ok(response)
    }

    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/transactions/by_hash/{}", hash)).await?)
    }

    pub async fn get_gas_price(&self) -> Result<GasFee, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/estimate_gas_price").await?)
    }

    pub async fn calculate_gas_limit(&self, input: &TransactionLoadInput) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let sequence = input.metadata.get_sequence()?;

        match &input.input_type {
            TransactionInputType::Transfer(asset)
            | TransactionInputType::Deposit(asset)
            | TransactionInputType::TransferNft(asset, _)
            | TransactionInputType::Account(asset, _) => {
                let payload = match &asset.id.token_id {
                    None => build_transfer_transaction_payload(&input.destination_address, &input.value),
                    Some(token_id) => build_token_transfer_transaction_payload(token_id, &input.destination_address, &input.value)?,
                };

                self.simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                    .await
            }
            TransactionInputType::Swap(_, _, swap_data) => match &swap_data.data.gas_limit {
                Some(gas_limit) => gas_limit.parse::<u64>().map_err(|_| "Invalid Aptos gas limit".into()),
                None => {
                    let payload: TransactionPayload = serde_json::from_str(&swap_data.data.data)?;
                    Ok(self
                        .simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                        .await
                        .unwrap_or(DEFAULT_SWAP_MAX_GAS_AMOUNT))
                }
            },
            TransactionInputType::Stake(_, stake_type) => {
                let payload = match stake_type {
                    StakeType::Stake(validator) => Some(build_stake_transaction_payload(&validator.id, &input.value)),
                    StakeType::Unstake(delegation) => Some(build_unstake_transaction_payload(&delegation.validator.id, &input.value)),
                    StakeType::Withdraw(delegation) => Some(build_withdraw_transaction_payload(&delegation.validator.id, &input.value)),
                    StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => None,
                };

                let payload = payload.ok_or("Unsupported Aptos stake type")?;
                self.simulate_transaction(&input.sender_address, sequence, payload, &input.gas_price.gas_price().to_string())
                    .await
            }
            TransactionInputType::Generic(_, _, _) => Ok(DEFAULT_MAX_GAS_AMOUNT),
            TransactionInputType::TokenApprove(_, _) | TransactionInputType::Perpetual(_, _) | TransactionInputType::Earn(_, _, _) => {
                Err("Unsupported Aptos transaction type".into())
            }
        }
    }

    pub async fn simulate_transaction(&self, sender: &str, sequence: u64, payload: TransactionPayload, gas_price: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let expiration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 1_000_000;

        let query = SimulateTransactionQuery {
            estimate_max_gas_amount: true,
            estimate_gas_unit_price: false,
            estimate_prioritized_gas_unit_price: false,
        };
        let path = build_path_with_query("/v1/transactions/simulate", &query)?;

        let simulation = TransactionSimulation {
            expiration_timestamp_secs: expiration.to_string(),
            gas_unit_price: gas_price.to_string(),
            max_gas_amount: DEFAULT_MAX_GAS_AMOUNT.to_string(),
            payload,
            sender: sender.to_string(),
            sequence_number: sequence.to_string(),
            signature: TransactionSignature::no_account(),
        };

        let response: Vec<Transaction> = self.client.post(&path, &simulation).await?;
        let transaction = response.into_iter().next().ok_or("No simulation result")?;

        transaction.gas_used.ok_or_else(|| "No gas used in simulation".into())
    }

    pub async fn get_validator_set(&self) -> Result<ValidatorSet, Box<dyn Error + Send + Sync>> {
        Ok(self.get_account_resource::<ValidatorSet>("0x1".to_string(), "0x1::stake::ValidatorSet").await?.data)
    }

    pub async fn get_staking_config(&self) -> Result<StakingConfig, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_account_resource::<StakingConfig>("0x1".to_string(), "0x1::staking_config::StakingConfig")
            .await?
            .data)
    }

    pub async fn get_delegation_pool_stake(&self, pool_address: &str, delegator_address: &str) -> Result<DelegationPoolStake, Box<dyn Error + Send + Sync>> {
        let view_request = serde_json::json!({
            "function": "0x1::delegation_pool::get_stake",
            "type_arguments": [],
            "arguments": [pool_address, delegator_address]
        });

        let (active, inactive, pending_inactive): (String, String, String) = self.client.post("/v1/view", &view_request).await?;

        Ok(DelegationPoolStake {
            active: BigUint::from_str(&active).unwrap_or_else(|_| BigUint::from(0u32)),
            inactive: BigUint::from_str(&inactive).unwrap_or_else(|_| BigUint::from(0u32)),
            pending_inactive: BigUint::from_str(&pending_inactive).unwrap_or_else(|_| BigUint::from(0u32)),
        })
    }

    pub async fn get_delegation_for_pool(&self, delegator_address: &str, pool_address: &str) -> Result<(String, DelegationPoolStake), Box<dyn Error + Send + Sync>> {
        let stake = self.get_delegation_pool_stake(pool_address, delegator_address).await?;
        Ok((pool_address.to_string(), stake))
    }

    pub async fn get_operator_commission_percentage(&self, pool_address: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let view_request = serde_json::json!({
            "function": "0x1::delegation_pool::operator_commission_percentage",
            "type_arguments": [],
            "arguments": [pool_address]
        });

        let result: Vec<String> = self.client.post("/v1/view", &view_request).await?;
        let commission_bps = result.first().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);

        Ok(commission_bps as f64 / 100.0)
    }

    pub async fn get_reconfiguration_state(&self) -> Result<ReconfigurationState, Box<dyn Error + Send + Sync>> {
        Ok(self
            .get_account_resource::<ReconfigurationState>("0x1".to_string(), "0x1::reconfiguration::Configuration")
            .await?
            .data)
    }

    pub async fn get_stake_lockup_secs(&self, pool_address: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let view_request = serde_json::json!({
            "function": "0x1::stake::get_lockup_secs",
            "type_arguments": [],
            "arguments": [pool_address]
        });

        let result: Vec<String> = self.client.post("/v1/view", &view_request).await?;
        let lockup_secs = result.first().and_then(|s| s.parse::<u64>().ok()).ok_or("Failed to parse lockup_secs")?;

        Ok(lockup_secs)
    }
}

#[cfg(feature = "rpc")]
mod chain_trait_impls {
    use super::*;
    use async_trait::async_trait;
    use chain_traits::{ChainAccount, ChainAddressStatus, ChainPerpetual};

    #[async_trait]
    impl<C: Client> ChainAccount for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainPerpetual for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainAddressStatus for AptosClient<C> {}
}
