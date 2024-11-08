// lib.rs

pub mod aptos;
pub mod bitcoin;
pub mod cosmos;
pub mod ethereum;
pub mod near;
pub mod solana;
pub mod sui;
pub mod ton;
pub mod tron;
pub mod xrp;

pub use self::aptos::client::AptosClient;
pub use self::bitcoin::client::BitcoinClient;
pub use self::cosmos::client::CosmosClient;
pub use self::ethereum::client::EthereumClient;
pub use self::near::client::NearClient;
pub use self::solana::client::SolanaClient;
pub use self::sui::client::SuiClient;
pub use self::ton::client::TonClient;
pub use self::tron::client::TronClient;
pub use self::xrp::client::XRPClient;

use async_trait::async_trait;
use primitives::{chain::Chain, Asset, Transaction};

use std::sync::Arc;

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
    async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> ChainBlockProvider for Arc<T>
where
    T: ChainBlockProvider + ?Sized,
{
    fn get_chain(&self) -> Chain {
        (**self).get_chain()
    }

    async fn get_latest_block(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_transactions(block_number).await
    }
}

#[async_trait]
impl<T: Send + Sync> ChainTokenDataProvider for Arc<T>
where
    T: ChainTokenDataProvider + ?Sized,
{
    async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<Asset, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_token_data(chain, token_id).await
    }
}
