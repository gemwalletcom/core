use crate::network::{AlienClient, AlienProvider};
use chain_traits::ChainBalances;
use gem_hypercore::rpc::client::HyperCoreClient;
use std::sync::Arc;

pub mod models;

pub use models::*;
use primitives::Chain;

#[derive(Debug, uniffi::Object)]
pub struct GemGateway {
    pub provider: Arc<dyn AlienProvider>,
}

impl GemGateway {
    pub async fn provider(&self, chain: Chain) -> Result<Arc<dyn ChainBalances>, GatewayError> {
        let url = self.provider.get_endpoint(chain).unwrap();
        let alien_client = AlienClient::new(url, self.provider.clone());
        match chain {
            Chain::HyperCore => Ok(Arc::new(HyperCoreClient::new(alien_client))),
            _ => Err(GatewayError::InvalidChain(chain.to_string())),
        }
    }
}

#[uniffi::export]
impl GemGateway {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn coin_balance(&self, chain: Chain, address: String) -> Result<GemAssetBalance, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_coin_balance(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into())
    }

    pub async fn token_balance(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_tokens_balance(address, token_ids)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into_iter().map(|b| b.into()).collect())
    }

    pub async fn get_stake_balance(&self, chain: Chain, address: String) -> Result<Option<GemAssetBalance>, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_stake_balance(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.map(|b| b.into()))
    }
}

#[derive(Debug, Clone, uniffi::Error, thiserror::Error)]
pub enum GatewayError {
    #[error("Invalid chain: {0}")]
    InvalidChain(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Balance not found: {0}")]
    BalanceNotFound(String),
}
