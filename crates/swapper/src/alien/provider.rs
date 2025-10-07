use super::{AlienError, Target};

use async_trait::async_trait;
use primitives::Chain;
use std::fmt::Debug;

pub type Data = Vec<u8>;

#[async_trait]
pub trait RpcProvider: Send + Sync + Debug {
    async fn request(&self, target: Target) -> Result<Data, AlienError>;
    async fn batch_request(&self, targets: Vec<Target>) -> Result<Vec<Data>, AlienError>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError>;
}
