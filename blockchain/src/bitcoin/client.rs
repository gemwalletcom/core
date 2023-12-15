use std::error::Error;

use crate::ChainProvider;
use async_trait::async_trait;
use chrono::Utc;
use primitives::{
    chain::Chain, transaction_utxo::TransactionInput, TransactionDirection, TransactionType,
};

use super::model::{Block, Status, Transaction};
use reqwest_middleware::ClientWithMiddleware;

pub struct BitcoinClient {
    chain: Chain,
    client: ClientWithMiddleware,
    url: String,
}

impl BitcoinClient {
    pub fn new(chain: Chain, client: ClientWithMiddleware, url: String) -> Self {
        Self { chain, client, url }
    }

    pub async fn get_status(&self) -> Result<Status, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/api/", self.url);
        Ok(self.client.get(url).send().await?.json::<Status>().await?)
    }

    pub async fn get_block(
        &self,
        block_number: i64,
        page: usize,
        limit: usize,
    ) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "{}/api/v2/block/{}?page={}&limit={}",
            self.url, block_number, page, limit
        );
        let block: Block = self.client.get(url).send().await?.json::<Block>().await?;
        Ok(block)
    }

    pub fn map_transaction(
        chain: Chain,
        transaction: &super::model::Transaction,
        _block_number: i64,
    ) -> Option<primitives::Transaction> {
        let inputs: Vec<TransactionInput> = transaction
            .vin
            .iter()
            .filter(|i| i.is_address)
            .map(|input| TransactionInput {
                address: input
                    .addresses
                    .clone()
                    .unwrap()
                    .first()
                    .unwrap()
                    .to_string(),
                value: input.value.clone(),
            })
            .collect();

        let outputs: Vec<TransactionInput> = transaction
            .vout
            .iter()
            .filter(|o| o.is_address)
            .map(|output| TransactionInput {
                address: output
                    .addresses
                    .clone()
                    .unwrap_or_default()
                    .first()
                    .unwrap()
                    .to_string(),
                value: output.value.clone(),
            })
            .collect();

        if inputs.is_empty() || outputs.is_empty() {
            return None;
        }

        let transaction = primitives::Transaction::new_with_utxo(
            transaction.txid.clone(),
            chain.as_asset_id(),
            None,
            None,
            None,
            TransactionType::Transfer,
            primitives::TransactionState::Confirmed,
            transaction.block_height.to_string(),
            0.to_string(),
            transaction.fees.clone(),
            chain.as_asset_id(),
            "0".to_string(),
            None,
            TransactionDirection::SelfTransfer,
            inputs,
            outputs,
            None,
            Utc::now(),
            //Utc.timestamp_opt(transaction.block_time, 0).unwrap(),
        );

        Some(transaction)
    }
}

#[async_trait]
impl ChainProvider for BitcoinClient {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let status = self.get_status().await?;
        Ok(status.blockbook.best_height)
    }

    async fn get_transactions(
        &self,
        block_number: i64,
    ) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let mut page: usize = 1;
        let limit: usize = 20;
        let mut transactions: Vec<Transaction> = Vec::new();
        loop {
            let block = self.get_block(block_number, page, limit).await?;
            transactions.extend(block.txs.clone());
            if block.page == block.total_pages {
                break;
            }
            page += 1;
        }
        let transactions = transactions
            .into_iter()
            .flat_map(|x| BitcoinClient::map_transaction(self.chain, &x, block_number))
            .collect::<Vec<primitives::Transaction>>();
        Ok(transactions)
    }
}

mod tests {

    #[test]
    fn test_finalize_with_address() {
        use super::*;

        let file_path = concat!(env!("CARGO_MANIFEST_DIR"), "/testdata/bitcoin/808497.json");
        let file = std::fs::File::open(file_path).expect("file should open read only");
        let json: serde_json::Value =
            serde_json::from_reader(file).expect("file should be proper JSON");

        // Test decoding json into struct
        let block: super::Block = serde_json::from_value(json).expect("Decoded into Block");
        assert_eq!(block.txs.len(), 1000);

        let chain = primitives::Chain::Bitcoin;
        let block_number = block.txs[0].block_height;

        // Test skipping coinbase tx
        let mut transaction = &block.txs[0];
        let mapped_tx = BitcoinClient::map_transaction(chain, transaction, block_number);
        assert!(mapped_tx.is_none());

        transaction = &block.txs[1];
        let mut mapped = BitcoinClient::map_transaction(chain, transaction, block_number).unwrap();

        assert!(mapped.from.is_empty());
        assert!(mapped.to.is_empty());
        assert_eq!(mapped.input_addresses().len(), 1);
        assert_eq!(mapped.output_addresses().len(), 2);
        assert_eq!(mapped.direction, TransactionDirection::SelfTransfer);

        let mut from = "3Q43ES6ENXr6h7c7FXDMixYWpZqr6imoJy".to_string();
        let to = "3MhDkWCqAvhTaCuD99tPsCz82SdgRtjeiB".to_string();
        let change = "35J3LCAoxvXFEtaJJd3PDEX9KpxQVHR5Ad".to_string();
        let unknown = "bc1qguju39vdhwm2eu4pxkzhuak2w3azvgf933vpmx".to_string();

        // Test unrelated tx
        let mut finalized = mapped.finalize(vec![unknown]);

        assert_eq!(finalized.from, "");
        assert_eq!(finalized.to, "");
        assert_eq!(finalized.direction, TransactionDirection::SelfTransfer);
        assert_eq!(finalized.value, "0");

        // Test Outgoing
        finalized = mapped.finalize(vec![from.clone(), change.clone()]);

        assert_eq!(finalized.from, from);
        assert_eq!(finalized.to, to);
        assert_eq!(finalized.direction, TransactionDirection::Outgoing);
        assert_eq!(finalized.value, "7360247");

        // Test Self
        finalized = mapped.finalize(vec![from.clone(), to.clone(), change.clone()]);

        assert_eq!(finalized.from, from);
        assert_eq!(finalized.to, to);
        assert_eq!(finalized.direction, TransactionDirection::SelfTransfer);
        assert_eq!(finalized.value, "321732570");

        // Test Incoming
        finalized = mapped.finalize(vec![change.clone()]);
        assert_eq!(finalized.from, from);
        assert_eq!(finalized.to, change);
        assert_eq!(finalized.direction, TransactionDirection::Incoming);
        assert_eq!(finalized.value, "314372323");

        // Test Outgoing without change
        transaction = &block.txs[2];
        mapped = BitcoinClient::map_transaction(chain, transaction, block_number).unwrap();
        from = "bc1qguju39vdhwm2eu4pxkzhuak2w3azvgf933vpmx".to_string();
        finalized = mapped.finalize(vec![from.clone()]);

        assert_eq!(finalized.from, from);
        assert_eq!(finalized.to, "bc1qyf7kvsmxp6vpgu7str5xz3kx58x8yc6j48pyct");
        assert_eq!(finalized.direction, TransactionDirection::Outgoing);
        assert_eq!(finalized.value, "34820917");
    }
}
