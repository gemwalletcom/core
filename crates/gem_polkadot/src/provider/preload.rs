use async_trait::async_trait;
use chain_traits::ChainTransactionLoad;
use num_bigint::BigInt;
use std::error::Error;

use gem_client::Client;
use primitives::{FeePriority, FeeRate, TransactionFee, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata, TransactionPreloadInput};

use crate::rpc::client::PolkadotClient;

#[async_trait]
impl<C: Client> ChainTransactionLoad for PolkadotClient<C> {
    async fn get_transaction_preload(&self, input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        let material = self.get_transaction_material().await?;
        let sender_balance = self.get_balance(input.sender_address).await?;

        Ok(TransactionLoadMetadata::Polkadot {
            sequence: sender_balance.nonce,
            genesis_hash: material.genesis_hash,
            block_hash: material.at.hash,
            block_number: material.at.height,
            spec_version: material.spec_version,
            transaction_version: material.tx_version,
            period: 64,
        })
    }

    async fn get_transaction_fee_from_data(&self, tx: String) -> Result<TransactionFee, Box<dyn Error + Sync + Send>> {
        let fee = self.estimate_fee(&tx).await?;
        Ok(TransactionFee::new_from_fee(BigInt::from(fee.partial_fee)))
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: input.default_fee(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        Ok(vec![FeeRate::regular(FeePriority::Normal, BigInt::from(1))])
    }
}
