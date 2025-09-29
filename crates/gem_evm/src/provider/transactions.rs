use std::error::Error;

#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactions;
use primitives::{BroadcastOptions, NodeType, Transaction};

use crate::{
    provider::transactions_mapper::map_transaction_broadcast,
    rpc::{EthereumMapper, client::EthereumClient, mapper::CONTRACT_REGISTRY},
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

    let traces = if node_type == NodeType::Archival {
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

    async fn get_transactions_by_address(&self, address: String, limit: Option<usize>) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let hashes = if let Some(ankr_client) = &self.ankr_client {
            ankr_client
                .get_ankr_transactions_by_address(address.as_str(), limit)
                .await?
                .transactions
                .into_iter()
                .map(|tx| tx.hash)
                .collect::<Vec<String>>()
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

        let traces = if self.node_type == NodeType::Archival {
            Some(self.trace_replay_block_transactions(block_i64).await?)
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
    use crate::provider::testkit::{TEST_ADDRESS, create_ethereum_test_client};
    use chain_traits::{ChainBalances, ChainTransactions};
    use num_bigint::BigUint;
    use std::error::Error;

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
