use std::error::Error;

use crate::rpc::model::{AddressDetails, Transaction};
use crate::typeshare::account::BitcoinAccount;
use crate::typeshare::block::{BitcoinBlock, BitcoinNodeInfo};
use crate::typeshare::transaction::BitcoinTransactionBroacastResult;

use super::model::{Block, Status};
use chain_traits::ChainTraits;
use gem_client::{Client, ContentType};
use primitives::chain::Chain;
use std::collections::HashMap;

#[derive(Debug)]
pub struct BitcoinClient<C: Client> {
    client: C,
    pub chain: Chain,
}

impl<C: Client> BitcoinClient<C> {
    pub fn new(client: C, chain: Chain) -> Self {
        Self { client, chain }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain
    }

    pub async fn get_status(&self) -> Result<Status, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/api/").await?)
    }

    pub async fn get_block(&self, block_number: i64, page: usize, limit: usize) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/block/{block_number}?page={page}&limit={limit}")).await?)
    }

    pub async fn get_address_details(&self, address: &str, limit: usize) -> Result<AddressDetails, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/address/{address}?pageSize={limit}&details=txs")).await?)
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/tx/{txid}")).await?)
    }

    pub async fn get_balance(&self, address: &str) -> Result<BitcoinAccount, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/address/{address}")).await?)
    }

    pub async fn get_block_info(&self, block_number: u64) -> Result<BitcoinBlock, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/block/{block_number}")).await?)
    }

    pub async fn get_node_info(&self) -> Result<BitcoinNodeInfo, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/api/").await?)
    }

    pub async fn broadcast_transaction(&self, data: String) -> Result<BitcoinTransactionBroacastResult, Box<dyn Error + Send + Sync>> {
        let headers = Some(HashMap::from([("Content-Type".to_string(), ContentType::TextPlain.as_str().to_string())]));
        Ok(self.client.post("/api/v2/sendtx/", &data, headers).await?)
    }
}

impl<C: Client> ChainTraits for BitcoinClient<C> {}
