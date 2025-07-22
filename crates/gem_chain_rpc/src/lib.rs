// lib.rs

pub mod algorand;
pub mod aptos;
pub mod bitcoin;
pub mod cardano;
pub mod cosmos;
pub mod ethereum;
pub mod hypercore;
pub mod near;
pub mod polkadot;
pub mod smartchain;
pub mod solana;
pub mod stellar;
pub mod sui;
pub mod ton;
pub mod tron;
pub mod xrp;

// Re-export all client implementations
pub use self::algorand::AlgorandProvider;
pub use self::aptos::AptosProvider;
pub use self::bitcoin::BitcoinProvider;
pub use self::cardano::CardanoProvider;
pub use self::cosmos::CosmosProvider;
pub use self::ethereum::EthereumProvider;
pub use self::hypercore::HyperCoreProvider;
pub use self::near::NearProvider;
pub use self::polkadot::PolkadotProvider;
pub use self::smartchain::SmartChainProvider;
pub use self::solana::SolanaProvider;
pub use self::stellar::StellarProvider;
pub use self::sui::SuiProvider;
pub use self::ton::TonProvider;
pub use self::tron::TronProvider;
pub use self::xrp::XRPProvider;

use async_trait::async_trait;
use primitives::{chain::Chain, Asset, AssetBalance, StakeValidator, Transaction};
use std::error::Error;

pub trait ChainProvider: ChainBlockProvider + ChainTokenDataProvider + ChainAssetsProvider + ChainTransactionsProvider + ChainStakeProvider {}
impl<T: ChainBlockProvider + ChainTokenDataProvider + ChainAssetsProvider + ChainTransactionsProvider + ChainStakeProvider> ChainProvider for T {}

#[async_trait]
pub trait ChainBlockProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>>;
    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>>;
}

impl dyn ChainBlockProvider {
    pub async fn get_transactions_in_blocks(&self, blocks: Vec<i64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = futures::future::try_join_all(blocks.iter().map(|block| self.get_transactions(*block)))
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<Transaction>>();
        Ok(transactions)
    }
}

impl dyn ChainProvider {
    pub async fn get_transactions_in_blocks(&self, blocks: Vec<i64>) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let transactions = futures::future::try_join_all(blocks.iter().map(|block| self.get_transactions(*block)))
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<Transaction>>();
        Ok(transactions)
    }
}

#[async_trait]
pub trait ChainTokenDataProvider: Send + Sync {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn Error + Send + Sync>> {
        Err("Not implemented".into())
    }
}

#[async_trait]
pub trait ChainAssetsProvider: Send + Sync {
    async fn get_assets_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainTransactionsProvider: Send + Sync {
    async fn get_transactions_by_address(&self, _address: String) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
pub trait ChainStakeProvider: Send + Sync {
    async fn get_validators(&self) -> Result<Vec<StakeValidator>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }

    async fn get_staking_apy(&self) -> Result<f64, Box<dyn Error + Send + Sync>> {
        Ok(0.0)
    }
}

pub struct MockChainBlockClient {
    pub chain: Chain,
}

impl MockChainBlockClient {
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }
}

#[async_trait]
impl ChainBlockProvider for MockChainBlockClient {
    fn get_chain(&self) -> Chain {
        self.chain
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn Error + Send + Sync>> {
        Ok(1)
    }

    async fn get_transactions(&self, _block_number: i64) -> Result<Vec<primitives::Transaction>, Box<dyn Error + Send + Sync>> {
        Ok(vec![])
    }
}

#[async_trait]
impl ChainAssetsProvider for MockChainBlockClient {
    // Default implementation returns empty vector
}

#[async_trait]
impl ChainTokenDataProvider for MockChainBlockClient {
    // Default implementation
}

#[async_trait]
impl ChainTransactionsProvider for MockChainBlockClient {
    // Default implementation returns empty vector
}

#[async_trait]
impl ChainStakeProvider for MockChainBlockClient {
    // Default implementation returns empty vector
}
