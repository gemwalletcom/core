use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    models::{BlockTransaction, SingleTransaction},
    provider::transaction_mapper::{map_block_transactions, map_signatures_transactions, map_transaction},
    rpc::{client::SolanaClient, constants::MISSING_BLOCKS_ERRORS},
};

#[async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        match self.get_block_transactions(block).await {
            Ok(block_transactions) => Ok(map_block_transactions(&block_transactions)),
            Err(error) => {
                if MISSING_BLOCKS_ERRORS.contains(&error.code) {
                    return Ok(vec![]);
                }
                Err(Box::new(error))
            }
        }
    }

    async fn get_transaction_by_hash(&self, hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let transaction = self
            .rpc_call::<SingleTransaction>("getTransaction", serde_json::json!([hash, { "encoding": "json", "maxSupportedTransactionVersion": 0 }]))
            .await?;
        let block_transaction = BlockTransaction {
            meta: transaction.meta,
            transaction: transaction.transaction,
        };
        Ok(map_transaction(&block_transaction, transaction.block_time))
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let limit = limit.unwrap_or(10);
        let signatures = self.get_signatures_for_address(&address, limit).await?;
        if signatures.is_empty() {
            return Ok(vec![]);
        }
        let signatures_ids = signatures.clone().iter().map(|x| x.signature.clone()).collect();
        let transactions = self.get_transactions(signatures_ids).await?;
        Ok(map_signatures_transactions(transactions, signatures))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_solana_test_client};
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_solana_get_transactions_by_block() {
        let client = create_solana_test_client();

        let latest_block = client.get_block_latest_number().await.unwrap();
        let transactions = client.get_transactions_by_block(latest_block).await.unwrap();

        println!("Latest block: {}, transactions count: {}", latest_block, transactions.len());
        assert!(latest_block > 0);
        assert!(!transactions.is_empty());
    }

    #[tokio::test]
    async fn test_solana_get_transactions_by_address() {
        let client = create_solana_test_client();
        let transactions = client.get_transactions_by_address(TransactionsRequest::new(TEST_ADDRESS.to_string())).await.unwrap();

        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
    }
}
