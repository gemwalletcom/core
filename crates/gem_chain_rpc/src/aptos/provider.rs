use std::error::Error;

use crate::{ChainBlockProvider, ChainTokenDataProvider};
use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetId, AssetType};
use num_bigint::BigUint;
use std::str::FromStr;
use chrono::Utc;
use primitives::{TransactionState, TransactionType};

use gem_aptos::model::{ResourceCoinInfo, DEPOSIT_EVENT};
use super::client::AptosClient;

pub struct AptosProvider {
    client: AptosClient,
}

impl AptosProvider {
    pub fn new(client: AptosClient) -> Self {
        Self { client }
    }
    
    pub fn map_transaction(&self, transaction: gem_aptos::model::Transaction, block_number: i64) -> Option<primitives::Transaction> {
        let events = transaction.clone().events.unwrap_or_default();

        if transaction.transaction_type == "user_transaction" && events.len() <= 4 {
            let deposit_event = events.iter().find(|x| x.event_type == DEPOSIT_EVENT)?;

            let asset_id = self.get_chain().as_asset_id();
            let state = if transaction.success {
                TransactionState::Confirmed
            } else {
                TransactionState::Failed
            };
            let to = &deposit_event.guid.account_address;
            let value = &deposit_event.get_amount()?;
            let gas_used = BigUint::from_str(transaction.gas_used.unwrap_or_default().as_str()).unwrap_or_default();
            let gas_unit_price = BigUint::from_str(transaction.gas_unit_price.unwrap_or_default().as_str()).unwrap_or_default();
            let fee = gas_used * gas_unit_price;

            let transaction = primitives::Transaction::new(
                transaction.hash,
                asset_id.clone(),
                transaction.sender.unwrap_or_default(),
                to.clone(),
                None,
                TransactionType::Transfer,
                state,
                block_number.to_string(),
                transaction.sequence_number.unwrap_or_default(),
                fee.to_string(),
                asset_id,
                value.clone(),
                None,
                None,
                Utc::now(),
            );
            return Some(transaction);
        }
        None
    }
}

#[async_trait]
impl ChainBlockProvider for AptosProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let ledger = self.client.get_ledger().await?;
        Ok(ledger.block_height.parse::<i64>().unwrap_or_default())
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = self.client.get_block_transactions(block_number).await?.transactions;
        let transactions = transactions
            .into_iter()
            .flat_map(|x| self.map_transaction(x, block_number))
            .collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}

#[async_trait]
impl ChainTokenDataProvider for AptosProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = token_id.split("::").collect();
        let address = parts.first().ok_or("Invalid token id")?;
        let resource = format!("0x1::coin::CoinInfo<{}>", token_id);
        let coin_info = self.client.get_resource::<ResourceCoinInfo>(address.to_string(), resource).await?.data;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            coin_info.name,
            coin_info.symbol,
            coin_info.decimals,
            AssetType::TOKEN,
        ))
    }
}
