use super::AlienError;
use super::AlienTarget;
use async_trait::async_trait;
use std::fmt::Debug;

pub type Data = Vec<u8>;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError>;
    async fn get_endpoint(&self, chain: primitives::Chain) -> Result<String, AlienError>;
}
