// lib.rs

mod aptos;
mod bnbchain;
use std::error::Error;

pub use self::bnbchain::client::BNBChainClient;
use async_trait::async_trait;

#[async_trait]
pub trait ChainProvider {
    async fn get_latest_block(&self) -> Result<i32, Box<dyn Error>>;
    async fn get_transactions(&self, block: i32) -> Result<Vec<i32>, Box<dyn Error>>;
}