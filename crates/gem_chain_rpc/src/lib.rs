// lib.rs

pub mod algorand;
pub mod aptos;
pub mod bitcoin;
pub mod cardano;
pub mod cosmos;
pub mod ethereum;
pub mod near;
pub mod polkadot;
pub mod solana;
pub mod stellar;
pub mod sui;
pub mod ton;
pub mod tron;
pub mod xrp;

pub use self::algorand::client::AlgorandClient;
pub use self::aptos::client::AptosClient;
pub use self::bitcoin::client::BitcoinClient;
pub use self::cardano::client::CardanoClient;
pub use self::cosmos::client::CosmosClient;
pub use self::ethereum::client::EthereumClient;
pub use self::near::client::NearClient;
pub use self::polkadot::client::PolkadotClient;
pub use self::solana::client::SolanaClient;
pub use self::stellar::client::StellarClient;
pub use self::sui::client::SuiClient;
pub use self::ton::client::TonClient;
pub use self::tron::client::TronClient;
pub use self::xrp::client::XRPClient;

use async_trait::async_trait;
use primitives::{chain::Chain, Asset, Transaction};

use std::error::Error;

pub trait ChainProvider: ChainBlockProvider + ChainTokenDataProvider {}
impl<T: ChainBlockProvider + ChainTokenDataProvider> ChainProvider for T {}

#[async_trait]
pub trait ChainBlockProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
    async fn get_latest_block(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
pub trait ChainTokenDataProvider: Send + Sync {
    async fn get_token_data(&self, _token_id: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        Err("Not implemented".into())
    }
}

// Temporary mock client until all providers are implemented
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
