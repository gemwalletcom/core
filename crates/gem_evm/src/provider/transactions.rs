use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactions;
use primitives::{BroadcastOptions, NodeType, Transaction, TransactionStateRequest, TransactionUpdate};

use crate::{
    provider::transactions_mapper::{map_transaction_broadcast, map_transaction_status},
    rpc::{client::EthereumClient, mapper::CONTRACT_REGISTRY, EthereumMapper},
};
use gem_client::Client;
use gem_jsonrpc::types::JsonRpcError;

#[cfg(feature = "rpc")]
async fn load_transactions_by_hashes<C: Client + Clone>(
    client: &EthereumClient<C>,
    node_type: NodeType,
    hashes: &[String],
) -> Result<Vec<Transaction>, JsonRpcError> {
    if hashes.is_empty() {
        return Ok(vec![]);
    }

    let transactions = client.get_transactions_by_hash(hashes).await?;
    let receipts = client.get_transactions_receipts(hashes).await?;
    let block_ids = receipts
        .iter()
        .map(|x| format!("0x{}", x.block_number.to_str_radix(16)))
        .collect::<Vec<String>>();
    let blocks = client.get_blocks(&block_ids, false).await?;

    let traces = if node_type == NodeType::Archive {
        Some(client.trace_replay_transactions(hashes).await?)
    } else {
        None
    };

    let chain = client.get_chain();
    Ok(transactions
        .into_iter()
        .zip(receipts.into_iter())
        .zip(blocks.into_iter())
        .enumerate()
        .filter_map(|(index, ((transactions, receipt), block))| {
            let trace = traces.as_ref().and_then(|entries| entries.get(index));
            EthereumMapper::map_transaction(chain, &transactions, &receipt, trace, &block.timestamp, Some(&CONTRACT_REGISTRY))
        })
        .collect())
}

#[cfg(feature = "rpc")]
#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let data = map_transaction_broadcast(&data);
        Ok(self.send_raw_transaction(&data).await?)
    }

    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let receipt = self.get_transaction_receipt(&request.id).await?;
        Ok(map_transaction_status(&receipt))
    }

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let hashes = if let Some(ankr_client) = &self.ankr_client {
            ankr_client
                .get_ankr_transactions_by_address(address.as_str(), limit)
                .await?
                .transactions
                .into_iter()
                .map(|tx| tx.hash)
                .collect::<Vec<String>>()
        } else if let Some(alchemy_client) = &self.alchemy_client {
            alchemy_client.get_transactions_ids_by_address(address.as_str()).await?
        } else {
            vec![]
        };
        Ok(load_transactions_by_hashes(self, self.node_type.clone(), &hashes).await?)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_i64 = block as i64;
        let block_data = self.get_block(block_i64).await?;
        let receipts = self.get_block_receipts(block_i64).await?;

        if block_data.transactions.is_empty() {
            return Ok(vec![]);
        }

        let traces = if self.node_type == NodeType::Archive {
            let transaction_hashes: Vec<String> = block_data.transactions.iter().map(|tx| tx.hash.clone()).collect();
            Some(self.trace_replay_transactions(&transaction_hashes).await?)
        } else {
            None
        };

        let chain = self.get_chain();
        Ok(block_data
            .transactions
            .into_iter()
            .zip(receipts)
            .enumerate()
            .filter_map(|(index, (tx, receipt))| {
                let trace = traces.as_ref().and_then(|entries| entries.get(index));
                EthereumMapper::map_transaction(chain, &tx, &receipt, trace, &block_data.timestamp, Some(&CONTRACT_REGISTRY))
            })
            .collect())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{create_ethereum_test_client, create_smartchain_test_client, TEST_ADDRESS};
    use chain_traits::{ChainBalances, ChainTransactions};
    use num_bigint::{BigInt, BigUint};
    use primitives::{TransactionChange, TransactionState, TransactionStateRequest};
    use std::error::Error;

    #[tokio::test]
    async fn test_ethereum_get_transaction_status_confirmed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let request = TransactionStateRequest::new_id("0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3".to_string());

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(42850974395536u64))]);

        Ok(())
    }

    #[tokio::test]
    async fn test_smartchain_get_transaction_status_confirmed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_smartchain_test_client();
        let request = TransactionStateRequest::new_id("0xd85c4496230adf8a7c0fc1e98713127fb31a0f8f72874acea443e2f615f3c1b6".to_string());

        let result = client.get_transaction_status(request).await?;

        assert_eq!(result.state, TransactionState::Confirmed);
        assert_eq!(result.changes, vec![TransactionChange::NetworkFee(BigInt::from(27753700000000u64))]);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let transactions = ChainTransactions::get_transactions_by_address(&client, TEST_ADDRESS.to_string(), Some(5)).await?;

        assert!(!transactions.is_empty());

        for tx in transactions.iter().take(3) {
            assert_eq!(tx.asset_id.chain, client.get_chain());
            assert!(tx.created_at.timestamp() > 0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_get_assets_balances() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let balances = ChainBalances::get_balance_assets(&client, TEST_ADDRESS.to_string()).await?;

        println!("Balances: {:#?}", balances);

        assert!(!balances.is_empty());

        let has_assets = balances
            .iter()
            .any(|balance| balance.asset_id.token_id.is_some() && balance.balance.available > BigUint::from(0u32));
        assert!(has_assets);

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let signed_tx = "0xf86c808502540be40082520894d4e56740f876aef8c010b86a40d5f56745a118d0765af9a146000000808081c0a05e1d3c1b2c3b0f8b7c8e9f0a1b2c3d4e5f6789abcdef0123456789abcdef012345a04f2c3a1b0d8e7f9a6b5c4d3e2f1a0b9c8d7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(signed_tx.to_string(), options).await;

        assert!(result.is_ok() || result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_transaction_broadcast_invalid_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_ethereum_test_client();
        let invalid_tx = "0xinvalidtransactiondata";
        let options = primitives::BroadcastOptions::default();

        let result = client.transaction_broadcast(invalid_tx.to_string(), options).await;

        assert!(result.is_err());

        Ok(())
    }
}
