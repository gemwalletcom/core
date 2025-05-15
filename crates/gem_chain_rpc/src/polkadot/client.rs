use std::error::Error;



use chrono::Utc;
use primitives::{Asset, Chain};
use reqwest_middleware::ClientWithMiddleware;

use super::model::{Block, BlockHeader, Extrinsic, ExtrinsicArguments, TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH, TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE};

pub struct PolkadotClient {
    url: String,
    client: ClientWithMiddleware,
}

impl PolkadotClient {
    pub fn new(client: ClientWithMiddleware, url: String) -> Self {
        Self { url, client }
    }

    pub async fn get_block_header(&self, block: &str) -> Result<BlockHeader, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}/header", self.url, block);
        Ok(self.client.get(url).send().await?.json::<BlockHeader>().await?)
    }

    pub async fn get_block(&self, block_number: i64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        let url = format!("{}/blocks/{}", self.url, block_number);
        Ok(self.client.get(url).send().await?.json::<Block>().await?)
    }

    fn map_transfer(&self, block: Block, transaction: Extrinsic, method: String, to_address: String, value: String) -> Option<primitives::Transaction> {
        if method != TRANSACTION_TYPE_TRANSFER_ALLOW_DEATH && method != TRANSACTION_TYPE_TRANSFER_KEEP_ALIVE {
            return None;
        }
        let from_address = transaction.signature?.signer.id.clone();
        let state = if transaction.success {
            primitives::TransactionState::Confirmed
        } else {
            primitives::TransactionState::Failed
        };
        let chain = Chain::Polkadot;
        Some(primitives::Transaction::new(
            transaction.hash.clone(),
            chain.as_asset_id(),
            from_address,
            to_address,
            None,
            primitives::TransactionType::Transfer,
            state,
            block.number,
            transaction.nonce.unwrap_or_default().clone(),
            transaction.info.partial_fee.unwrap_or("0".to_string()),
            chain.as_asset_id(),
            value,
            None,
            None,
            Utc::now(),
        ))
    }

    pub fn map_transaction(&self, block: Block, transaction: Extrinsic) -> Vec<Option<primitives::Transaction>> {
        match &transaction.args.clone() {
            ExtrinsicArguments::Transfer(transfer) => {
                vec![self.map_transfer(
                    block,
                    transaction.clone(),
                    transaction.method.method.clone(),
                    transfer.dest.id.clone(),
                    transfer.value.clone(),
                )]
            }
            ExtrinsicArguments::Transfers(transfers) => transfers
                .calls
                .iter()
                .map(|x| {
                    self.map_transfer(
                        block.clone(),
                        transaction.clone(),
                        x.method.method.clone(),
                        x.args.dest.id.clone(),
                        x.args.value.clone(),
                    )
                })
                .collect(),
            _ => vec![],
        }
    }
    
    pub fn get_chain(&self) -> Chain {
        Chain::Polkadot
    }
    
    pub async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        unimplemented!()
    }
}
