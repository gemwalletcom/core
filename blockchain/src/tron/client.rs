use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use num_traits::Num;
use primitives::{chain::Chain, TransactionType, TransactionState, AssetId};
use chrono::Utc;
use num_bigint::BigUint;

use super::{model::{Block, BlockTransactions, BlockTransactionsInfo, Transaction, TransactionReceiptData}, address::TronAddress};
use reqwest_middleware::ClientWithMiddleware;

const TRANSFER_CONTRACT: &str = "TransferContract";
const TRIGGER_SMART_CONTRACT: &str = "TriggerSmartContract";

pub struct TronClient {
    url: String,
    client: ClientWithMiddleware,
}

impl TronClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self {
            url,
            client,
        }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/wallet/getblock", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<Block>()
            .await?;
        Ok(response)
    }

    pub async fn get_block_tranactions(&self, block: i64) -> Result<BlockTransactions, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/walletsolidity/getblockbynum?num={}", self.url, block);
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BlockTransactions>()
            .await?;
        Ok(response)
    }

    pub async fn get_block_tranactions_reciepts(&self, block: i64) -> Result<BlockTransactionsInfo, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/walletsolidity/gettransactioninfobyblocknum?num={}", self.url, block);
        let response = self.client
            .get(&url)
            .send()
            .await?
            .json::<BlockTransactionsInfo>()
            .await?;
        Ok(response)
    }

    pub fn map_transaction(&self, transaction: Transaction, receipt: TransactionReceiptData) -> Option<primitives::Transaction> {
      
        if let (Some(value), Some(contract_result)) = (transaction.raw_data.contract.first().cloned(), transaction.ret.first().cloned()) {
            let state: TransactionState = if contract_result.contract_ret.clone() == "SUCCESS" { TransactionState::Confirmed } else { TransactionState::Failed };
            let fee = receipt.fee.unwrap_or_default().to_string();
            if value.contract_type == TRANSFER_CONTRACT && !transaction.ret.is_empty() {
                let from = TronAddress::from_hex(value.parameter.value.owner_address.unwrap_or_default().as_str()).unwrap_or_default();
                let to = TronAddress::from_hex(value.parameter.value.to_address.unwrap_or_default().as_str()).unwrap_or_default();
                
                let transaction = primitives::Transaction::new(
                    transaction.tx_id,
                    self.get_chain().as_asset_id(),
                    from,
                    to,
                    None,
                    TransactionType::Transfer,
                    state,
                    receipt.block_number.to_string(),
                    0.to_string(),
                    fee,
                    self.get_chain().as_asset_id(),
                    value.parameter.value.amount.unwrap_or_default().to_string(),
                    None,
                    None,
                    Utc::now()
                );
                return Some(transaction)
            }
            let logs = receipt.log.unwrap_or_default();
            // TRC20 transfers
            if value.contract_type == TRIGGER_SMART_CONTRACT && logs.len() == 1 && logs.first().unwrap().topics.clone().unwrap_or_default().len() == 3 && logs.first().unwrap().topics.clone().unwrap_or_default().first().unwrap() == "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef" {
                let log = logs.first().unwrap();
                let from_string = format!("41{}", log.topics.clone().unwrap_or_default()[1].clone().chars().skip(24).collect::<String>());
                let to_string = format!("41{}", log.topics.clone().unwrap_or_default()[2].clone().chars().skip(24).collect::<String>());
                let token_id = TronAddress::from_hex(value.parameter.value.contract_address.unwrap().as_str()).unwrap_or_default();
                let from = TronAddress::from_hex(from_string.as_str()).unwrap_or_default();
                let to = TronAddress::from_hex(to_string.as_str()).unwrap_or_default();
                let value = BigUint::from_str_radix(&log.data.clone().unwrap_or_default(), 16).unwrap();
                let asset_id = AssetId {chain: self.get_chain(), token_id: Some(token_id)};

                let transaction = primitives::Transaction::new(
                    transaction.tx_id,
                    asset_id,
                    from,
                    to,
                    None,
                    TransactionType::Transfer,
                    state,
                    receipt.block_number.to_string(),
                    0.to_string(),
                    fee,
                    self.get_chain().as_asset_id(),
                    value.to_string(),
                    None,
                    None,
                    Utc::now()
                );
                
                return Some(transaction)

            }
        }
        None
   }
}

#[async_trait]
impl ChainProvider for TronClient {

    fn get_chain(&self) -> Chain {
        Chain::Tron
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_block().await?;
        Ok(block.block_header.raw_data.number)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block_tranactions(block_number).await?;
        let transactions = block.transactions.unwrap_or_default();
        let reciepts = self.get_block_tranactions_reciepts(block_number).await?;

        let transactions = transactions.into_iter().zip(reciepts.iter()).filter_map(|(transaction, receipt)| {
            self.map_transaction(transaction, receipt.clone())
        }).collect::<Vec<primitives::Transaction>>();

        Ok(transactions)
    }
}