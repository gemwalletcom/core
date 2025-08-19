use std::error::Error;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "rpc")]
use gem_client::Client;
#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainAccount, ChainPerpetual, ChainStaking, ChainTraits, ChainState, ChainTransactions, ChainToken, ChainPreload};
use primitives::{chain::Chain, Asset, FeePriorityValue, TransactionStateRequest, TransactionUpdate};

use super::{
    model::Balance,
};
use crate::models::staking::SuiStakeDelegation;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub struct SuiClient {
    client: JsonRpcClient,
}

#[cfg(feature = "rpc")]
pub struct SuiClient<C: Client> {
    pub client: C,
}

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
impl SuiClient {
    pub fn new(client: JsonRpcClient) -> Self {
        Self { client }
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> SuiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn get_chain(&self) -> Chain {
        Chain::Sui
    }

    pub async fn get_balance(&self, address: String) -> Result<Balance, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/accounts/{}/balance", address)).await?)
    }

    pub async fn get_all_balances(&self, address: String) -> Result<Vec<Balance>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/accounts/{}/balances", address)).await?)
    }

    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }

    pub async fn get_stake_delegations(&self, address: String) -> Result<Vec<SuiStakeDelegation>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/accounts/{}/stakes", address)).await?)
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> ChainTraits for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainAccount for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainPerpetual for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainStaking for SuiClient<C> {}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client> ChainState for SuiClient<C> {
    async fn get_chain_id(&self) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        Ok("sui".to_string())
    }

    async fn get_block_number(&self) -> Result<u64, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_fee_rates(&self) -> Result<Vec<FeePriorityValue>, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }
}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client> ChainTransactions for SuiClient<C> {
    async fn transaction_broadcast(&self, _data: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_transaction_status(&self, _request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }
}

#[cfg(feature = "rpc")]
impl<C: Client> ChainToken for SuiClient<C> {}

#[cfg(feature = "rpc")]
impl<C: Client> ChainPreload for SuiClient<C> {}
