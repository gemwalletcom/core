use crate::gateway::models::asset::GemAsset;
use crate::network::{AlienClient, AlienProvider, jsonrpc_client_with_chain};
use chain_traits::ChainTraits;
use gem_algorand::rpc::client::AlgorandClient;
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::client::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_near::rpc::client::NearClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::client::SuiClient;
use gem_polkadot::rpc::client::PolkadotClient;
use gem_solana::rpc::client::SolanaClient;
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::client::TronClient;
use gem_xrp::rpc::client::XRPClient;
use std::sync::Arc;

pub mod models;

pub use models::*;
use primitives::{chain_cosmos::CosmosChain, BitcoinChain, Chain, ChartPeriod};

#[derive(Debug, uniffi::Object)]
pub struct GemGateway {
    pub provider: Arc<dyn AlienProvider>,
}

impl GemGateway {
    pub async fn provider(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let url = self.provider.get_endpoint(chain).unwrap();
        let alien_client = AlienClient::new(url.clone(), self.provider.clone());
        match chain {
            Chain::HyperCore => Ok(Arc::new(HyperCoreClient::new(alien_client))),
            Chain::Bitcoin | Chain::BitcoinCash | Chain::Litecoin | Chain::Doge => {
                Ok(Arc::new(BitcoinClient::new(alien_client, BitcoinChain::from_chain(chain).unwrap())))
            }
            Chain::Cardano => Ok(Arc::new(CardanoClient::new(alien_client))),
            Chain::Stellar => Ok(Arc::new(StellarClient::new(alien_client))),
            Chain::Sui => Ok(Arc::new(SuiClient::new(jsonrpc_client_with_chain(self.provider.clone(), chain)))),
            Chain::Xrp => Ok(Arc::new(XRPClient::new(alien_client))),
            Chain::Algorand => Ok(Arc::new(AlgorandClient::new(alien_client))),
            Chain::Near => Ok(Arc::new(NearClient::new(jsonrpc_client_with_chain(self.provider.clone(), chain)))),
            Chain::Aptos => Ok(Arc::new(AptosClient::new(alien_client))),
            Chain::Cosmos | Chain::Osmosis | Chain::Celestia | Chain::Thorchain | Chain::Injective | Chain::Sei | Chain::Noble => {
                Ok(Arc::new(CosmosClient::new(CosmosChain::from_chain(chain).unwrap(), alien_client, url)))
            }
            Chain::Ton => Ok(Arc::new(TonClient::new(alien_client))),
            Chain::Tron => Ok(Arc::new(TronClient::new(alien_client))),
            Chain::Polkadot => Ok(Arc::new(PolkadotClient::new(alien_client))),
            Chain::Solana => Ok(Arc::new(SolanaClient::new(jsonrpc_client_with_chain(self.provider.clone(), chain)))),
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
        let balance = self
            .provider(chain)
            .await?
            .get_balance_coin(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into())
    }

    pub async fn get_balance_tokens(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        let balance = self
            .provider(chain)
            .await?
            .get_balance_tokens(address, token_ids)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.into_iter().map(|b| b.into()).collect())
    }

    pub async fn get_balance_staking(&self, chain: Chain, address: String) -> Result<Option<GemAssetBalance>, GatewayError> {
        let balance = self
            .provider(chain)
            .await?
            .get_balance_staking(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(balance.map(|b| b.into()))
    }

    pub async fn get_staking_validators(&self, chain: Chain, apy: Option<f64>) -> Result<Vec<GemDelegationValidator>, GatewayError> {
        let provider = self.provider(chain).await?;

        let validators = provider
            .get_staking_validators(apy)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(validators.into_iter().map(|v| v.into()).collect())
    }

    pub async fn get_staking_delegations(&self, chain: Chain, address: String) -> Result<Vec<GemDelegationBase>, GatewayError> {
        let delegations = self
            .provider(chain)
            .await?
            .get_staking_delegations(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(delegations.into_iter().map(|d| d.into()).collect())
    }

    pub async fn transaction_broadcast(&self, chain: Chain, data: String) -> Result<String, GatewayError> {
        let hash = self
            .provider(chain)
            .await?
            .transaction_broadcast(data)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(hash)
    }

    pub async fn get_transaction_status(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<GemTransactionUpdate, GatewayError> {
        let status = self
            .provider(chain)
            .await?
            .get_transaction_status(request.into())
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(status.into())
    }

    pub async fn get_chain_id(&self, chain: Chain) -> Result<String, GatewayError> {
        let chain_id = self
            .provider(chain)
            .await?
            .get_chain_id()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(chain_id)
    }

    pub async fn get_block_number(&self, chain: Chain) -> Result<u64, GatewayError> {
        let block_number = self
            .provider(chain)
            .await?
            .get_block_number()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(block_number)
    }

    pub async fn get_fee_rates(&self, chain: Chain) -> Result<Vec<GemFeeRate>, GatewayError> {
        let fees = self
            .provider(chain)
            .await?
            .get_fee_rates()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(fees.into_iter().map(|f| f.into()).collect())
    }

    pub async fn get_utxos(&self, chain: Chain, address: String) -> Result<Vec<GemUTXO>, GatewayError> {
        let utxos = self
            .provider(chain)
            .await?
            .get_utxos(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(utxos.into_iter().map(|u| u.into()).collect())
    }

    pub async fn get_transaction_preload(&self, chain: Chain, input: GemTransactionPreloadInput) -> Result<GemTransactionPreload, GatewayError> {
        let preload = self
            .provider(chain)
            .await?
            .get_transaction_preload(input.into())
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(preload.into())
    }

    pub async fn get_transaction_load(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<GemTransactionData, GatewayError> {
        let load_data = self
            .provider(chain)
            .await?
            .get_transaction_load(input.clone().into())
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        Ok(models::transaction::map_transaction_load_data(load_data, &input))
    }

    pub async fn get_positions(&self, chain: Chain, address: String) -> Result<GemPerpetualPositionsSummary, GatewayError> {
        let positions = self
            .provider(chain)
            .await?
            .get_positions(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(positions.into())
    }

    pub async fn get_perpetuals_data(&self, chain: Chain) -> Result<Vec<GemPerpetualData>, GatewayError> {
        let data = self
            .provider(chain)
            .await?
            .get_perpetuals_data()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        Ok(data.into_iter().map(|d| d.into()).collect())
    }

    pub async fn get_candlesticks(&self, chain: Chain, symbol: String, period: String) -> Result<Vec<GemChartCandleStick>, GatewayError> {
        let chart_period = ChartPeriod::new(period).ok_or_else(|| GatewayError::ParseError("Invalid chart period".to_string()))?;
        let candlesticks = self
            .provider(chain)
            .await?
            .get_candlesticks(symbol, chart_period)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        Ok(candlesticks.into_iter().map(|c| c.into()).collect())
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<GemAsset, GatewayError> {
        Ok(self
            .provider(chain)
            .await?
            .get_token_data(token_id)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?
            .into())
    }

    pub async fn get_is_token_address(&self, chain: Chain, token_id: String) -> Result<bool, GatewayError> {
        Ok(self.provider(chain).await?.get_is_token_address(&token_id))
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
