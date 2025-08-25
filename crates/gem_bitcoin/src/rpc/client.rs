use std::error::Error;

use crate::models::account::BitcoinAccount;
use crate::models::block::{BitcoinBlock, BitcoinNodeInfo, Block, Status};
use crate::models::fee::BitcoinFeeResult;
use crate::models::transaction::{AddressDetails, BitcoinTransactionBroacastResult, BitcoinUTXO, Transaction};
use chain_traits::{ChainPerpetual, ChainStaking, ChainToken, ChainTraits};
use gem_client::{Client, ContentType};
use primitives::{chain::Chain, BitcoinChain};
use std::collections::HashMap;

#[derive(Debug)]
pub struct BitcoinClient<C: Client> {
    client: C,
    pub chain: BitcoinChain,
}

impl<C: Client> BitcoinClient<C> {
    pub fn new(client: C, chain: BitcoinChain) -> Self {
        Self { client, chain }
    }

    pub fn get_chain(&self) -> Chain {
        self.chain.get_chain()
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

    pub async fn get_utxos(&self, address: &str) -> Result<Vec<BitcoinUTXO>, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/api/v2/utxo/{address}")).await?)
    }

    pub async fn get_fee_priority(&self, blocks: i32) -> Result<String, Box<dyn Error + Send + Sync>> {
        let result: BitcoinFeeResult = self.client.get(&format!("/api/v2/estimatefee/{blocks}")).await?;
        Ok(result.result)
    }
}

impl<C: Client> ChainStaking for BitcoinClient<C> {}

impl<C: Client> ChainPerpetual for BitcoinClient<C> {}

impl<C: Client> ChainToken for BitcoinClient<C> {}

impl<C: Client> ChainTraits for BitcoinClient<C> {}

impl<C: Client> chain_traits::ChainProvider for BitcoinClient<C> {
    fn get_chain(&self) -> primitives::Chain {
        self.chain.get_chain()
    }
}
