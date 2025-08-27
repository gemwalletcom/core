use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::{ChainTransactionLoad, ChainTransactions};
#[cfg(feature = "rpc")]
use gem_client::Client;
use primitives::{
    FeePriority, FeeRate, GasPriceType, Transaction, TransactionFee, TransactionInputType, TransactionLoadData, TransactionLoadInput, TransactionLoadMetadata,
    TransactionPreloadInput, TransactionStateRequest, TransactionUpdate,
};

use crate::provider::transactions_mapper;
use crate::rpc::client::SuiClient;

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactionLoad for SuiClient<C> {
    async fn get_transaction_preload(&self, _input: TransactionPreloadInput) -> Result<TransactionLoadMetadata, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadMetadata::None)
    }

    async fn get_transaction_load(&self, input: TransactionLoadInput) -> Result<TransactionLoadData, Box<dyn Error + Sync + Send>> {
        Ok(TransactionLoadData {
            fee: TransactionFee::default(),
            metadata: input.metadata,
        })
    }

    async fn get_transaction_fee_rates(&self, _input_type: TransactionInputType) -> Result<Vec<FeeRate>, Box<dyn Error + Sync + Send>> {
        let gas_price = self.get_gas_price().await?;
        Ok(vec![FeeRate::new(FeePriority::Normal, GasPriceType::regular(gas_price))])
    }
}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for SuiClient<C> {
    async fn transaction_broadcast(&self, _data: String) -> Result<String, Box<dyn std::error::Error + Sync + Send>> {
        unimplemented!()
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn std::error::Error + Sync + Send>> {
        let transaction = self.get_transaction(request.id).await?;
        let state = match transaction.effects.status.status.as_str() {
            "success" => primitives::TransactionState::Confirmed,
            "failure" => primitives::TransactionState::Reverted,
            _ => primitives::TransactionState::Pending,
        };
        Ok(TransactionUpdate::new_state(state))
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        let checkpoint = self.get_transactions_by_block(block).await?;
        let mut transactions = Vec::new();

        for tx_id in checkpoint.transactions {
            if let Ok(tx) = self.get_transaction(tx_id).await {
                if let Some(mapped_tx) = transactions_mapper::map_transaction(tx) {
                    transactions.push(mapped_tx);
                }
            }
        }

        Ok(transactions)
    }

    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self
            .get_transactions_by_address(address)
            .await?
            .data
            .into_iter()
            .flat_map(transactions_mapper::map_transaction)
            .collect())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::*;
    use chain_traits::ChainState;

    #[tokio::test]
    async fn test_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_sui_test_client();
        let latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(latest_block - 1).await?;
        println!("Transactions in block {}: {}", latest_block - 1, transactions.network_total_transactions);
        Ok(())
    }
}
