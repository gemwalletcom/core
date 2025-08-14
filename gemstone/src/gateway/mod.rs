use crate::network::AlienProvider;
use std::sync::Arc;

pub mod models;

pub use models::*;
use primitives::Chain;

#[derive(Debug, uniffi::Object)]
pub struct GemGateway {
    pub chain: Chain,
    //pub rpc_provider: Arc<dyn AlienProvider>,
}

#[uniffi::export]
impl GemGateway {
    #[uniffi::constructor]
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }

    pub async fn coin_balance(&self, address: String) -> Result<AssetBalanceWrapper, GatewayError> {
        Ok(AssetBalanceWrapper {
            asset_id: self.chain.as_asset_id().to_string(),
            balance: BalanceWrapper {
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
    }

    pub async fn token_balance(&self, address: String, token_ids: Vec<String>) -> Result<Vec<AssetBalanceWrapper>, GatewayError> {
        let balances = token_ids
            .into_iter()
            .map(|token_id| AssetBalanceWrapper {
                asset_id: token_id,
                balance: BalanceWrapper {
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

    pub async fn get_stake_balance(&self, address: String) -> Result<Option<AssetBalanceWrapper>, GatewayError> {
        Ok(Some(AssetBalanceWrapper {
            asset_id: self.chain.as_asset_id().to_string(),
            balance: BalanceWrapper {
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
