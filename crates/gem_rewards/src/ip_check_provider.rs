use std::error::Error;

use async_trait::async_trait;

use crate::model::IpCheckResult;

#[async_trait]
pub trait IpCheckProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>>;
}
