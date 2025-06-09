use super::{AlienError, AlienTarget};

use async_trait::async_trait;
use primitives::Chain;
use std::fmt::Debug;

pub type Data = Vec<u8>;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait AlienProvider: Send + Sync + Debug {
    async fn request(&self, target: AlienTarget) -> Result<Data, AlienError>;
    async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError>;
}
