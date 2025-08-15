use alloy_primitives::hex;
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_solana::model::BigUint;
use std::error::Error;

use crate::{ChainAssetsProvider, ChainBlockProvider, ChainStakeProvider, ChainTokenDataProvider, ChainTransactionsProvider, SmartChainProvider};
use gem_evm::{
    erc20::{decode_abi_string, decode_abi_uint8, IERC20},
    ethereum_address_checksum,
    rpc::{alchemy::AlchemyClient, ankr::AnkrClient, EthereumClient, EthereumMapper},
};
use primitives::{Asset, AssetBalance, AssetId, Chain, EVMChain, NodeType, StakeValidator, Transaction};

pub struct EthereumProvider {
    client: EthereumClient,
    node_type: NodeType,
    assets_provider: Box<dyn ChainAssetsProvider>,
    transactions_provider: Box<dyn ChainTransactionsProvider>,
}

impl EthereumProvider {
    pub fn new(
        client: EthereumClient,
        node_type: NodeType,
        assets_provider: Box<dyn ChainAssetsProvider>,
        transactions_provider: Box<dyn ChainTransactionsProvider>,
    ) -> Self {
        Self {
            client,
            node_type,
            assets_provider,
            transactions_provider,
        }
    }
}

#[async_trait]
impl ChainBlockProvider for EthereumProvider {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get_latest_block().await?)
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.client.get_block(block_number).await?;
        let reciepts = self.client.get_block_receipts(block_number).await?;
        match self.node_type {
            NodeType::Default => Ok(EthereumMapper::map_transactions(self.get_chain(), block, reciepts, None)),
            NodeType::Archive => {
                let traces = self.client.trace_replay_block_transactions(block_number).await?;
                Ok(EthereumMapper::map_transactions(self.get_chain(), block, reciepts, Some(traces)))
            }
        }
    }
}

#[async_trait]
impl ChainTokenDataProvider for EthereumProvider {
    async fn get_token_data(&self, token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        let name: String = self
            .client
            .eth_call(token_id.as_str(), &hex::encode_prefixed(IERC20::nameCall {}.abi_encode()))
            .await?;
        let symbol: String = self
            .client
            .eth_call(token_id.as_str(), &hex::encode_prefixed(IERC20::symbolCall {}.abi_encode()))
            .await?;
        let decimals: String = self
            .client
            .eth_call(token_id.as_str(), &hex::encode_prefixed(IERC20::decimalsCall {}.abi_encode()))
            .await?;

        let name_value = decode_abi_string(&name)?;
        let symbol_value = decode_abi_string(&symbol)?;
        let decimals_value = decode_abi_uint8(&decimals)?;

        let asset_type = self
            .get_chain()
            .default_asset_type()
            .ok_or_else(|| format!("No default asset type for chain {:?}", self.get_chain()))?;

        Ok(Asset::new(
            AssetId::from_token(self.get_chain(), &token_id),
            name_value,
            symbol_value,
            decimals_value as i32,
            asset_type,
        ))
    }
}

#[async_trait]
impl ChainAssetsProvider for EthereumProvider {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.assets_provider.get_assets_balances(address).await
    }
}

#[async_trait]
impl ChainTransactionsProvider for EthereumProvider {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.transactions_provider.get_transactions_by_address(address).await
    }
}

#[async_trait]
impl ChainStakeProvider for EthereumProvider {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        match self.client.chain {
            EVMChain::SmartChain => SmartChainProvider::new(self.client.clone()).get_validators().await,
            _ => Ok(vec![]),
        }
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        match self.client.chain {
            EVMChain::SmartChain => SmartChainProvider::new(self.client.clone()).get_staking_apy().await,
            _ => Ok(0.0),
        }
    }
}

// AlchemyClient

#[async_trait]
impl ChainAssetsProvider for AlchemyClient {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let response = self.get_token_balances(&address).await?;
        let balances = response
            .data
            .tokens
            .into_iter()
            .filter(|x| x.token_balance != BigUint::from(0u32))
            .filter_map(|x| {
                ethereum_address_checksum(&x.token_address).ok().map(|from| AssetBalance::new(
                    AssetId::from_token(self.chain.to_chain(), &from),
                    x.token_balance.to_string(),
                ))
            })
            .collect();
        Ok(balances)
    }
}

#[async_trait]
impl ChainTransactionsProvider for AlchemyClient {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_transactions_by_address(address.as_str()).await?)
    }
}

// AnkrClient

#[async_trait]
impl ChainTransactionsProvider for AnkrClient {
    async fn get_transactions_by_address(&self, address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_transactions_by_address(address.as_str(), 25).await?)
    }
}

#[async_trait]
impl ChainAssetsProvider for AnkrClient {
    async fn get_assets_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let response = self.get_token_balances(&address).await?;
        let balances = response
            .assets
            .into_iter()
            .filter(|x| x.contract_address.is_some())
            .filter_map(|x| {
                ethereum_address_checksum(&x.contract_address?).ok().map(|from| AssetBalance::new(
                    AssetId::from_token(self.chain.to_chain(), &from),
                    x.balance_raw_integer.to_string(),
                ))
            })
            .collect();
        Ok(balances)
    }
}
