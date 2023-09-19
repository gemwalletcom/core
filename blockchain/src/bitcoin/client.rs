use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use primitives::chain::Chain;

use super::model::Block;
use reqwest_middleware::ClientWithMiddleware;

pub struct BitcoinClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl BitcoinClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self {
            chain,
            client,
            url,
        }
    }

    pub async fn get_block(&self) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api", self.url);
        let response = self.client
            .get(url)
            .send()
            .await?
            .json::<Block>()
            .await?;
        return Ok(response);
    }

    pub fn map_transaction(&self, _transaction: super::model::Transaction) -> Option<primitives::Transaction> {
        // let transaction = primitives::Transaction{
        //     id: "".to_string(),
        //     hash: transaction.tx_id,
        //     asset_id: AssetId::from_chain(self.get_chain()),
        //     from,
        //     to,
        //     contract: None,
        //     transaction_type: TransactionType::Transfer,
        //     state,
        //     block_number: receipt.block_number as i32,
        //     sequence: 0,
        //     fee: receipt.fee.unwrap_or_default().to_string(),
        //     fee_asset_id: AssetId::from_chain(self.get_chain()),
        //     value: value.parameter.value.amount.unwrap_or_default().to_string(),
        //     memo: None,
        //     direction: TransactionDirection::SelfTransfer,
        //     created_at: Utc::now().naive_utc(),
        //     updated_at: Utc::now().naive_utc(),
        // };
       return None;
   }
}

#[async_trait]
impl ChainProvider for BitcoinClient {

    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.get_block().await?;
        Ok(block.blockbook.best_height)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        //TODO: Implement
        // let transactions = transactions.into_iter()
        //     .flat_map(|x| self.map_transaction(x, block_number))
        //     .collect::<Vec<primitives::Transaction>>();
        // Ok(transactions) 
        Ok(vec![])
    }
}