use crate::models::*;
use crate::network::{AlienClient, AlienProvider, jsonrpc_client_with_chain};
use chain_traits::ChainTraits;
use gem_algorand::rpc::client::AlgorandClient;
use gem_aptos::rpc::client::AptosClient;
use gem_bitcoin::rpc::client::BitcoinClient;
use gem_cardano::rpc::client::CardanoClient;
use gem_cosmos::rpc::client::CosmosClient;
use gem_evm::rpc::EthereumClient;
use gem_hypercore::rpc::client::HyperCoreClient;
use gem_near::rpc::client::NearClient;
use gem_polkadot::rpc::client::PolkadotClient;
use gem_solana::rpc::client::SolanaClient;
use gem_stellar::rpc::client::StellarClient;
use gem_sui::rpc::client::SuiClient;
use gem_ton::rpc::client::TonClient;
use gem_tron::rpc::{client::TronClient, trongrid::client::TronGridClient};
use gem_xrp::rpc::client::XRPClient;
use std::sync::Arc;

use primitives::{BitcoinChain, Chain, ChartPeriod, EVMChain, chain_cosmos::CosmosChain};

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait GemGatewayEstimateFee: Send + Sync {
    async fn get_fee(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<Option<GemTransactionLoadFee>, GatewayError>;
    async fn get_fee_data(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<Option<String>, GatewayError>;
}

#[uniffi::export(with_foreign)]
pub trait GemPreferences: Send + Sync {
    fn get(&self, key: String) -> Result<Option<String>, GatewayError>;
    fn set(&self, key: String, value: String) -> Result<(), GatewayError>;
    fn remove(&self, key: String) -> Result<(), GatewayError>;
}

struct PreferencesWrapper {
    preferences: Arc<dyn GemPreferences>,
}

impl primitives::Preferences for PreferencesWrapper {
    fn get(&self, key: String) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.get(key).map_err(Into::into)
    }

    fn set(&self, key: String, value: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.set(key, value).map_err(Into::into)
    }

    fn remove(&self, key: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.preferences.remove(key).map_err(Into::into)
    }
}

#[derive(uniffi::Object)]
pub struct GemGateway {
    pub provider: Arc<dyn AlienProvider>,
    pub preferences: Arc<dyn GemPreferences>,
    pub secure_preferences: Arc<dyn GemPreferences>,
}

impl std::fmt::Debug for GemGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GemGateway")
            .field("provider", &"<AlienProvider>")
            .field("preferences", &"<GemPreferences>")
            .field("secure_preferences", &"<GemPreferences>")
            .finish()
    }
}

impl GemGateway {
    pub async fn provider(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let url = self.provider.get_endpoint(chain).unwrap();
        self.provider_with_url(chain, url).await
    }

    pub async fn provider_with_url(&self, chain: Chain, url: String) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        let alien_client = AlienClient::new(url.clone(), self.provider.clone());
        match chain {
            Chain::HyperCore => {
                let preferences = Arc::new(PreferencesWrapper {
                    preferences: self.preferences.clone(),
                });
                let secure_preferences = Arc::new(PreferencesWrapper {
                    preferences: self.secure_preferences.clone(),
                });
                Ok(Arc::new(HyperCoreClient::new_with_preferences(alien_client, preferences, secure_preferences)))
            }
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
                Ok(Arc::new(CosmosClient::new(CosmosChain::from_chain(chain).unwrap(), alien_client)))
            }
            Chain::Ton => Ok(Arc::new(TonClient::new(alien_client))),
            Chain::Tron => Ok(Arc::new(TronClient::new(
                alien_client.clone(),
                TronGridClient::new(alien_client.clone(), String::new()),
            ))),
            Chain::Polkadot => Ok(Arc::new(PolkadotClient::new(alien_client))),
            Chain::Solana => Ok(Arc::new(SolanaClient::new(jsonrpc_client_with_chain(self.provider.clone(), chain)))),
            Chain::Ethereum
            | Chain::Arbitrum
            | Chain::SmartChain
            | Chain::Polygon
            | Chain::Optimism
            | Chain::Base
            | Chain::AvalancheC
            | Chain::OpBNB
            | Chain::Fantom
            | Chain::Gnosis
            | Chain::Manta
            | Chain::Blast
            | Chain::ZkSync
            | Chain::Linea
            | Chain::Mantle
            | Chain::Celo
            | Chain::World
            | Chain::Sonic
            | Chain::Abstract
            | Chain::Berachain
            | Chain::Ink
            | Chain::Unichain
            | Chain::Hyperliquid
            | Chain::Monad => Ok(Arc::new(EthereumClient::new(
                jsonrpc_client_with_chain(self.provider.clone(), chain),
                EVMChain::from_chain(chain).unwrap(),
            ))),
        }
    }
}

#[async_trait::async_trait]
impl GemGatewayEstimateFee for GemGateway {
    async fn get_fee(&self, _chain: Chain, _input: GemTransactionLoadInput) -> Result<Option<GemTransactionLoadFee>, GatewayError> {
        Ok(None)
    }

    async fn get_fee_data(&self, _chain: Chain, _input: GemTransactionLoadInput) -> Result<Option<String>, GatewayError> {
        Ok(None)
    }
}

#[uniffi::export]
impl GemGateway {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferences>, secure_preferences: Arc<dyn GemPreferences>) -> Self {
        Self {
            provider,
            preferences,
            secure_preferences,
        }
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

    pub async fn transaction_broadcast(&self, chain: Chain, data: String, options: GemBroadcastOptions) -> Result<String, GatewayError> {
        let hash = self
            .provider(chain)
            .await?
            .transaction_broadcast(data, options.into())
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
            .get_block_latest_number()
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(block_number)
    }

    pub async fn get_fee_rates(&self, chain: Chain, input: GemTransactionInputType) -> Result<Vec<GemFeeRate>, GatewayError> {
        let fees = self
            .provider(chain)
            .await?
            .get_transaction_fee_rates(input.into())
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

    pub async fn get_address_status(&self, chain: Chain, address: String) -> Result<Vec<GemAddressStatus>, GatewayError> {
        let status = self
            .provider(chain)
            .await?
            .get_address_status(address)
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(status.into_iter().collect())
    }

    pub async fn get_transaction_preload(&self, chain: Chain, input: GemTransactionPreloadInput) -> Result<GemTransactionLoadMetadata, GatewayError> {
        let metadata = self
            .provider(chain)
            .await?
            .get_transaction_preload(input.into())
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
        Ok(metadata.into())
    }

    pub async fn get_fee(
        &self,
        chain: Chain,
        input: GemTransactionLoadInput,
        provider: Arc<dyn GemGatewayEstimateFee>,
    ) -> Result<Option<GemTransactionLoadFee>, GatewayError> {
        let fee = provider.get_fee(chain, input.clone()).await?;
        if let Some(fee) = fee {
            return Ok(Some(fee));
        }
        if let Some(fee_data) = provider.get_fee_data(chain, input.clone()).await? {
            let data = self
                .provider(chain)
                .await?
                .get_transaction_fee_from_data(fee_data)
                .await
                .map_err(|e| GatewayError::NetworkError(e.to_string()))?;
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    pub async fn get_transaction_load(
        &self,
        chain: Chain,
        input: GemTransactionLoadInput,
        provider: Arc<dyn GemGatewayEstimateFee>,
    ) -> Result<GemTransactionData, GatewayError> {
        let fee = self.get_fee(chain, input.clone(), provider.clone()).await?;

        let load_data = self
            .provider(chain)
            .await?
            .get_transaction_load(input.clone().into())
            .await
            .map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        let data = if let Some(fee) = fee { load_data.new_from(fee.into()) } else { load_data };

        Ok(GemTransactionData {
            fee: data.fee.into(),
            metadata: data.metadata.into(),
        })
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
        let chart_period = ChartPeriod::new(period).unwrap();
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

    pub async fn get_node_status(&self, chain: Chain, url: &str) -> Result<GemNodeStatus, GatewayError> {
        let start_time = std::time::Instant::now();
        let provider = self.provider_with_url(chain, url.to_string()).await?;

        let (chain_id, latest_block_number) =
            futures::try_join!(provider.get_chain_id(), provider.get_block_latest_number()).map_err(|e| GatewayError::NetworkError(e.to_string()))?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        Ok(GemNodeStatus {
            chain_id,
            latest_block_number,
            latency_ms,
        })
    }
}

#[derive(Debug, Clone, uniffi::Error)]
pub enum GatewayError {
    NetworkError(String),
}

impl std::fmt::Display for GatewayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for GatewayError {}
