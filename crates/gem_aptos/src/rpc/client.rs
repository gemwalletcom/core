use std::error::Error;

use gem_client::Client;
use primitives::chain::Chain;
use primitives::{AssetSubtype, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};
use serde::{de::DeserializeOwned, Serialize};

use crate::models::{
    Account, Block, GasFee, Ledger, Resource, ResourceData, Transaction, TransactionPayload, TransactionResponse, TransactionSignature, TransactionSimulation,
};
pub type AccountResource<T> = Resource<T>;

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
        Chain::Aptos
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

    pub async fn get_account_resource<T: Serialize + DeserializeOwned>(
        &self,
        address: String,
        resource: &str,
    ) -> Result<Option<AccountResource<T>>, Box<dyn Error + Send + Sync>> {
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

    pub async fn submit_transaction(&self, data: &str) -> Result<TransactionResponse, Box<dyn Error + Send + Sync>> {
        let json_value: serde_json::Value = serde_json::from_str(data)?;
        let response = self
            .client
            .post::<serde_json::Value, TransactionResponse>("/v1/transactions", &json_value, None)
            .await?;

        if let Some(message) = &response.message {
            return Err(Box::new(std::io::Error::other(message.clone())));
        }

        Ok(response)
    }

    pub async fn get_resources(&self, address: &str) -> Result<Vec<Resource<ResourceData>>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/accounts/{}/resources", address)).await?)
    }

    pub async fn get_transaction_by_hash(&self, hash: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/v1/transactions/by_hash/{}", hash)).await?)
    }

    pub async fn get_gas_price(&self) -> Result<GasFee, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/v1/estimate_gas_price").await?)
    }

    pub async fn calculate_gas_limit(&self, input: &TransactionLoadInput) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let sequence = match &input.metadata {
            TransactionLoadMetadata::Aptos { sequence } => *sequence,
            _ => return Err("Invalid metadata type for Aptos".into()),
        };

        match &input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
                let asset_type = if asset.id.token_id.is_none() {
                    AssetSubtype::NATIVE
                } else {
                    AssetSubtype::TOKEN
                };

                match asset_type {
                    AssetSubtype::NATIVE => {
                        // For native transfers, simulate the transaction
                        let simulated = self
                            .simulate_transaction(
                                &input.sender_address,
                                &input.destination_address,
                                &sequence.to_string(),
                                &input.value,
                                &input.gas_price.gas_price().to_string(),
                                1500,
                            )
                            .await?;
                        Ok(simulated.gas_used.unwrap_or(1500))
                    }
                    AssetSubtype::TOKEN => Ok(1500),
                }
            }
            TransactionInputType::Swap(_, _) => Ok(1500),
            TransactionInputType::Stake(_, _) => Err("Aptos does not support staking".into()),
            TransactionInputType::TokenApprove(_, _) => Ok(1500),
            TransactionInputType::Generic(_, _, _) => Ok(1500),
            TransactionInputType::Perpetual(_, _) => unimplemented!(),
        }
    }

    pub async fn simulate_transaction(
        &self,
        sender: &str,
        recipient: &str,
        sequence: &str,
        value: &str,
        gas_price: &str,
        max_gas_amount: u64,
    ) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let expiration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 1_000_000;

        let simulation = TransactionSimulation {
            expiration_timestamp_secs: expiration.to_string(),
            gas_unit_price: gas_price.to_string(),
            max_gas_amount: max_gas_amount.to_string(),
            payload: TransactionPayload {
                function: "0x1::aptos_account::transfer".to_string(),
                type_arguments: vec![],
                arguments: vec![recipient.to_string(), value.to_string()],
                payload_type: "entry_function_payload".to_string(),
            },
            sender: sender.to_string(),
            sequence_number: sequence.to_string(),
            signature: TransactionSignature {
                signature_type: "no_account_signature".to_string(),
                public_key: None,
                signature: None,
            },
        };

        let response: Vec<Transaction> = self.client.post("/v1/transactions/simulate", &simulation, None).await?;
        response.into_iter().next().ok_or_else(|| "No simulation result".into())
    }
}

#[cfg(feature = "rpc")]
mod chain_trait_impls {
    use super::*;
    use async_trait::async_trait;
    use chain_traits::{ChainAccount, ChainPerpetual, ChainStaking};

    #[async_trait]
    impl<C: Client> ChainStaking for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainAccount for AptosClient<C> {}

    #[async_trait]
    impl<C: Client> ChainPerpetual for AptosClient<C> {}
}
