// lib.rs

mod aptos;
mod bnbchain;
mod solana;
mod ethereum;
mod ton;
mod cosmos;
mod tron;

pub use self::bnbchain::client::BNBChainClient;
pub use self::solana::client::SolanaClient;
pub use self::ethereum::client::EthereumClient;
pub use self::ton::client::TonClient;
pub use self::cosmos::client::CosmosClient;
pub use self::tron::client::TronClient;

use async_trait::async_trait;
use primitives::{chain::Chain, Transaction};

use std::sync::Arc;

#[async_trait]
pub trait ChainProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
    async fn get_latest_block(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_transactions(&self, block_number: i64) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> ChainProvider for Arc<T>
where
    T: ChainProvider + ?Sized,
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