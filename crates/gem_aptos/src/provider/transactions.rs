use async_trait::async_trait;
use chain_traits::ChainTransactions;
use serde::Deserialize;
use std::error::Error;

use gem_client::Client;
use primitives::BroadcastOptions;

use super::transactions_mapper::map_transaction_broadcast;
use crate::{provider::transactions_mapper::map_transactions, rpc::client::AptosClient};

#[async_trait]
impl<C: Client> ChainTransactions for AptosClient<C> {
    async fn transaction_broadcast(&self, data: String, _options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        let bcs_bytes = extract_bcs_bytes(&data)?;
        let result = self.submit_transaction(bcs_bytes).await?;
        map_transaction_broadcast(&result)
    }

    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_block_transactions(block).await?.transactions))
    }

    async fn get_transactions_by_address(&self, _address: String, _limit: Option<usize>) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(map_transactions(self.get_transactions_by_address(_address).await?))
    }
}

#[derive(Deserialize)]
struct BcsWrapper {
    bcs: String,
    #[serde(rename = "bcsEncoding")]
    bcs_encoding: Option<String>,
}

fn extract_bcs_bytes(data: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let wrapper = serde_json::from_str::<BcsWrapper>(data).map_err(|err| {
        Box::new(std::io::Error::other(format!(
            "Unsupported Aptos submit payload: {err}"
        ))) as Box<dyn Error + Send + Sync>
    })?;

    if let Some(encoding) = wrapper.bcs_encoding.as_deref()
        && encoding != "hex" {
            return Err(Box::new(std::io::Error::other(format!(
                "Unsupported Aptos BCS encoding: {encoding}"
            ))));
        }

    decode_bcs_hex(&wrapper.bcs)
}

fn decode_bcs_hex(data: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Err(Box::new(std::io::Error::other("Empty Aptos BCS payload")));
    }
    let hex_value = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes =
        hex::decode(hex_value).map_err(|err| std::io::Error::other(format!("Invalid Aptos BCS hex: {err}")))?;
    Ok(bytes)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use crate::provider::testkit::{TEST_ADDRESS, create_aptos_test_client};
    use chain_traits::{ChainState, ChainTransactions};

    #[tokio::test]
    async fn test_aptos_get_transactions_by_block() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let _latest_block = client.get_block_latest_number().await?;
        let transactions = client.get_transactions_by_block(100000).await?;
        println!("Transactions in block 100000: {}", transactions.len());
        Ok(())
    }

    #[tokio::test]
    async fn test_aptos_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();
        let transactions = client.get_transactions_by_address(TEST_ADDRESS.to_string()).await?;
        println!("Address: {}, transactions count: {}", TEST_ADDRESS, transactions.len());
        Ok(())
    }
}
