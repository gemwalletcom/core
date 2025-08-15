use crate::network::{AlienClient, AlienProvider};
use chain_traits::ChainTraits;
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
    pub async fn provider(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
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

    pub async fn get_balance_coin(&self, chain: Chain, address: String) -> Result<GemAssetBalance, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_balance_coin(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into())
    }

    pub async fn get_balance_tokens(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_balance_tokens(address, token_ids)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into_iter().map(|b| b.into()).collect())
    }

    pub async fn get_balance_staking(&self, chain: Chain, address: String) -> Result<Option<GemAssetBalance>, GatewayError> {
        let provider = self.provider(chain).await?;
        let balance = provider
            .get_balance_staking(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.map(|b| b.into()))
    }

    // staking
    pub async fn get_staking_validators(&self, chain: Chain) -> Result<Vec<GemDelegationValidator>, GatewayError> {
        let provider = self.provider(chain).await?;
        let validators = provider.get_staking_validators().await.map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(validators.into_iter().map(|v| v.into()).collect())
    }

    pub async fn get_staking_delegations(&self, chain: Chain, address: String) -> Result<Vec<GemDelegationBase>, GatewayError> {
        let provider = self.provider(chain).await?;
        let delegations = provider
            .get_staking_delegations(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(delegations.into_iter().map(|d| d.into()).collect())
    }

    pub async fn transaction_broadcast(&self, chain: Chain, data: String) -> Result<String, GatewayError> {
        let provider = self.provider(chain).await?;
        let hash = provider
            .transaction_broadcast(data)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(hash)
    }

    pub async fn get_transaction_status(&self, chain: Chain, hash: String) -> Result<String, GatewayError> {
        let provider = self.provider(chain).await?;
        let status = provider
            .get_transaction_status(hash)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(status)
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
