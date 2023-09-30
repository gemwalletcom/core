use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use primitives::{chain::Chain, TransactionType, TransactionState, TransactionDirection};
use chrono::Utc;

use super::{model::{Block, BlockTransactions, BlockTransactionsInfo, Transaction, TransactionReceiptData}, address::TronAddress};
use reqwest_middleware::ClientWithMiddleware;

const TRANSFER_CONTRACT: &str = "TransferContract";

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
            if value.contract_type == TRANSFER_CONTRACT && !transaction.ret.is_empty() {
                let from = TronAddress::from_hex(value.parameter.value.owner_address.unwrap_or_default().as_str()).unwrap_or_default();
                let to = TronAddress::from_hex(value.parameter.value.to_address.unwrap_or_default().as_str()).unwrap_or_default();
                let state: TransactionState = if contract_result.contract_ret.clone() == "SUCCESS" { TransactionState::Confirmed } else { TransactionState::Failed };
                
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
                    receipt.fee.unwrap_or_default().to_string(),
                    self.get_chain().as_asset_id(),
                    value.parameter.value.amount.unwrap_or_default().to_string(),
                    None,
                    TransactionDirection::SelfTransfer,
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