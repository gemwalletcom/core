use super::{client::EthereumClient, mapper::EthereumMapper};
use crate::erc20::IERC20;
use alloy_primitives::hex;
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_chain_rpc::{ChainAssetsProvider, ChainBlockProvider, ChainTokenDataProvider};
use primitives::{chain::Chain, Asset, AssetBalance, AssetId};
use std::error::Error;

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

        let name_value = IERC20::nameCall::abi_decode_returns(&hex::decode(name)?).unwrap();
        let symbol_value = IERC20::symbolCall::abi_decode_returns(&hex::decode(symbol)?).unwrap();
        let decimals_value = IERC20::decimalsCall::abi_decode_returns(&hex::decode(decimals)?).unwrap();

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
