use super::model::MayanChain;
use crate::{
    SwapperError,
    alien::{RpcProvider, Target},
};
use std::sync::Arc;

pub struct MayanPrice {
    base_url: String,
    provider: Arc<dyn RpcProvider>,
}

impl MayanPrice {
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider>) -> Self {
        Self { base_url, provider }
    }

    pub async fn get_chains(&self) -> Result<Vec<MayanChain>, SwapperError> {
        let url = format!("{}/v3/chains", self.base_url);
        let target = Target::get(&url);
        let response = self.provider.request(target).await?;
        let chains: Vec<MayanChain> = serde_json::from_slice(&response.data).map_err(SwapperError::from)?;
        Ok(chains)
    }
}
