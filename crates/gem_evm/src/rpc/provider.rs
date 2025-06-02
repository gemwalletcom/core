use std::error::Error;

use super::{client::EthereumClient, mapper::EthereumMapper};
use async_trait::async_trait;
use gem_chain_rpc::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use hex::FromHex;
use primitives::{chain::Chain, Asset, AssetBalance, AssetId};

pub struct EthereumProvider {
    client: EthereumClient,
    assets_provider: Box<dyn ChainAssetsProvider>,
}

impl EthereumProvider {
    pub fn new(client: EthereumClient, assets_provider: Box<dyn ChainAssetsProvider>) -> Self {
        Self { client, assets_provider }
    }
}

#[async_trait]
impl ChainBlockProvider for EthereumProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_latest_block().await?;
        Ok(block)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        let transactions_reciepts = self.client.get_block_receipts(block_number).await?;
        let transactions = block.transactions;

        let transactions = transactions
            .into_iter()
            .zip(transactions_reciepts.iter())
            .filter_map(|(transaction, receipt)| EthereumMapper::map_transaction(self.get_chain(), &transaction, receipt, block.timestamp.clone()))
            .collect::<Vec<primitives::Transaction>>();

        return Ok(transactions);
    }
}

#[async_trait]
impl ChainTokenDataProvider for EthereumProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_NAME).await?;
        let symbol: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_SYMBOL).await?;
        let decimals: String = self.client.eth_call(token_id.as_str(), super::client::FUNCTION_ERC20_DECIMALS).await?;

        let name_bytes = Vec::from_hex(name)?;
        let symbol_bytes = Vec::from_hex(symbol)?;
        let decimals_bytes = Vec::from_hex(decimals)?;

        let name_value = String::from_utf8(name_bytes.clone()).unwrap_or_default();
        let symbol_value = String::from_utf8(symbol_bytes.clone()).unwrap_or_default();
        let decimals_value: u8 = decimals_bytes.first().copied().unwrap_or_default();

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            name_value,
            symbol_value,
            decimals_value as i32,
            self.get_chain().default_asset_type().unwrap(),
        ))
    }
}

#[async_trait]
impl ChainAssetsProvider for EthereumProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.assets_provider.get_assets_balances(address).await
    }
}
