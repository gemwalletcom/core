use async_trait::async_trait;
use chain_traits::ChainPreload;
use std::error::Error;
use num_bigint::BigInt;

use gem_client::Client;
use primitives::{TransactionLoadData, TransactionLoadInput, TransactionPreload, TransactionPreloadInput, TransactionFee};
use primitives::transaction_load::TransactionLoadMetadata;

use crate::rpc::client::PolkadotClient;


#[async_trait]
impl<C: Client> ChainPreload for PolkadotClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionPreload, Box<dyn Error + Sync + Send>> {
        let block = self.get_block_head().await?;
        Ok(TransactionPreload::builder()
            .block_number(block.number as i64)
            .build())
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        let transaction_material = self.get_transaction_material().await?;
        let sender_balance = self.get_balance(input.sender_address.clone()).await?;
        let nonce = sender_balance.nonce;
        let _ = self.get_balance(input.destination_address.clone()).await?;
        
        let spec_version = transaction_material.spec_version;
        let transaction_version = transaction_material.tx_version;
        let block_number = transaction_material.at.height;
        
        // For now, use a hardcoded fee of 0.016 DOT (16,000,000,000 Planck units)
        // TODO: Implement proper fee estimation once transaction encoding is stabilized
        let fee = BigInt::from(160_000_000u128);
        
        let fee = TransactionFee {
            fee,
            gas_price: input.gas_price.gas_price.clone(),
            gas_limit: BigInt::from(1u64),
            options: std::collections::HashMap::new(),
        };

        Ok(TransactionLoadData {
            fee,
            metadata: TransactionLoadMetadata::Polkadot {
                sequence: nonce,
                genesis_hash: transaction_material.genesis_hash,
                block_hash: transaction_material.at.hash,
                block_number,
                spec_version,
                transaction_version,
                period: 64,
            },
        })
    }
}