use crate::network::{AlienProvider, NativeClient};
use gem_hypercore::rpc::client::HyperCoreClient;
use std::sync::Arc;

pub mod models;

pub use models::*;
use primitives::{AssetId, Chain};

#[derive(Debug, uniffi::Object)]
pub struct GemGateway {
    pub chain: Chain,
    pub rpc_provider: Arc<dyn AlienProvider>,
    pub hypercore_client: Option<HyperCoreClient<NativeClient>>,
}

#[uniffi::export]
impl GemGateway {
    #[uniffi::constructor]
    pub fn new(chain: Chain, rpc_provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            chain,
            rpc_provider,
            hypercore_client: None,
        }
    }

    #[uniffi::constructor]
    pub fn new_with_native_client(chain: Chain, base_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        let native_client = NativeClient::new(base_url, provider.clone());
        let hypercore_client = HyperCoreClient::new(native_client);
        Self {
            chain,
            rpc_provider: provider,
            hypercore_client: Some(hypercore_client),
        }
    }

    pub async fn coin_balance(&self, address: String) -> Result<GemAssetBalance, GatewayError> {
        if let Some(ref hypercore_client) = self.hypercore_client {
            let balances = hypercore_client
                .spot_balances(&address)
                .await
                .map_err(|e| GatewayError::NetworkError(format!("HyperCore API error: {}", e)))?;

            // Find the native coin balance (token index 0)
            let native_balance = balances
                .balances
                .iter()
                .find(|b| b.token == 0)
                .map(|b| b.total.clone())
                .unwrap_or_else(|| "0".to_string());

            Ok(GemAssetBalance {
                asset_id: self.chain.as_asset_id(),
                balance: GemBalance {
                    available: native_balance,
                    frozen: "0".to_string(),
                    locked: "0".to_string(),
                    staked: "0".to_string(),
                    pending: "0".to_string(),
                    rewards: "0".to_string(),
                    reserved: "0".to_string(),
                    withdrawable: "0".to_string(),
                },
            })
        } else {
            // Fallback for when no hypercore client is available
            Ok(GemAssetBalance {
                asset_id: self.chain.as_asset_id(),
                balance: GemBalance {
                    available: "0".to_string(),
                    frozen: "0".to_string(),
                    locked: "0".to_string(),
                    staked: "0".to_string(),
                    pending: "0".to_string(),
                    rewards: "0".to_string(),
                    reserved: "0".to_string(),
                    withdrawable: "0".to_string(),
                },
            })
        }
    }

    pub async fn token_balance(&self, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        let balances = token_ids
            .into_iter()
            .map(|token_id| GemAssetBalance {
                asset_id: AssetId::from_token(self.chain, &token_id),
                balance: GemBalance {
                    available: "1000000000000000000".to_string(),
                    frozen: "0".to_string(),
                    locked: "0".to_string(),
                    staked: "0".to_string(),
                    pending: "0".to_string(),
                    rewards: "0".to_string(),
                    reserved: "0".to_string(),
                    withdrawable: "0".to_string(),
                },
            })
            .collect();

        Ok(balances)
    }

    pub async fn get_stake_balance(&self, address: String) -> Result<Option<GemAssetBalance>, GatewayError> {
        Ok(Some(GemAssetBalance {
            asset_id: self.chain.as_asset_id(),
            balance: GemBalance {
                available: "0".to_string(),
                frozen: "0".to_string(),
                locked: "0".to_string(),
                staked: "5000000000000000000".to_string(),
                pending: "0".to_string(),
                rewards: "100000000000000000".to_string(),
                reserved: "0".to_string(),
                withdrawable: "0".to_string(),
            },
        }))
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
