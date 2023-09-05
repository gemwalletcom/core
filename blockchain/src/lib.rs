// lib.rs

mod aptos;
mod bnbchain;

pub use self::bnbchain::client::BNBChainClient;
use async_trait::async_trait;
use primitives::{chain::Chain, Transaction};

use std::sync::Arc;

#[async_trait]
pub trait ChainProvider: Send + Sync {
    fn get_chain(&self) -> Chain;
    async fn get_latest_block(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_transactions(&self, block_number: i32) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>>;
}

#[async_trait]
impl<T: Send + Sync> ChainProvider for Arc<T>
where
    T: ChainProvider + ?Sized,
{
    fn get_chain(&self) -> Chain {
        (**self).get_chain()
    }

    async fn get_latest_block(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_latest_block().await
    }

    async fn get_transactions(&self, block_number: i32) -> Result<Vec<Transaction>, Box<dyn std::error::Error + Send + Sync>> {
        (**self).get_transactions(block_number).await
    }
}