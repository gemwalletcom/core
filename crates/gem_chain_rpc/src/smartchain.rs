use alloy_primitives::hex;
use async_trait::async_trait;
use std::error::Error;

use crate::ChainStakeProvider;
use gem_bsc::{stake_hub, HUB_READER_ADDRESS};
use gem_evm::rpc::EthereumClient;
use primitives::StakeValidator;

pub struct SmartChainProvider {
    client: EthereumClient,
}

impl SmartChainProvider {
    pub fn new(client: EthereumClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ChainStakeProvider for SmartChainProvider {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        let call_data = hex::encode_prefixed(stake_hub::encode_validators_call(0, 100));

        let result: String = self.client.eth_call(HUB_READER_ADDRESS, &call_data).await?;
        let result_bytes = hex::decode(&result)?;
        let bsc_validators = stake_hub::decode_validators_return(&result_bytes)?;

        let validators = bsc_validators
            .into_iter()
            .filter(|v| !v.jailed)
            .map(|v| StakeValidator::new(v.operator_address, v.moniker))
            .collect();

        Ok(validators)
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let call_data = hex::encode_prefixed(stake_hub::encode_validators_call(0, 100));

        let result: String = self.client.eth_call(HUB_READER_ADDRESS, &call_data).await?;
        let result_bytes = hex::decode(&result)?;
        let bsc_validators = stake_hub::decode_validators_return(&result_bytes)?;

        if bsc_validators.is_empty() {
            return Ok(0.0);
        }

        let max_apy = bsc_validators
            .iter()
            .filter(|v| !v.jailed)
            .map(|v| v.apy)
            .max()
            .unwrap_or(0);

        // Convert from basis points (10000 = 100%) to percentage
        Ok(max_apy as f64 / 100.0)
    }
}
