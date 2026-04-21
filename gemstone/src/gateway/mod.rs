mod chain_factory;
mod error;
mod preferences;

pub use chain_factory::ChainClientFactory;
pub use error::GatewayError;
use error::map_network_error;
#[cfg(test)]
pub use preferences::EmptyPreferences;
pub use preferences::GemPreferences;
pub(crate) use preferences::PreferencesWrapper;

use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::api_client::GemApiClient;
use crate::models::*;
use chain_traits::ChainTraits;
use std::future::Future;
use std::sync::Arc;
use yielder::Yielder;

use primitives::{AssetId, Chain, ChartPeriod, ScanAddressTarget, ScanTransactionPayload, TransactionPreloadInput};

#[uniffi::export(with_foreign)]
#[async_trait::async_trait]
pub trait GemGatewayEstimateFee: Send + Sync {
    async fn get_fee(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<Option<GemTransactionLoadFee>, GatewayError>;
    async fn get_fee_data(&self, chain: Chain, input: GemTransactionLoadInput) -> Result<Option<String>, GatewayError>;
}

#[derive(uniffi::Object)]
pub struct GemGateway {
    pub api_client: GemApiClient,
    chain_factory: Arc<ChainClientFactory>,
    yielder: Yielder,
}

impl std::fmt::Debug for GemGateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GemGateway").field("api_client", &self.api_client).finish()
    }
}

impl GemGateway {
    async fn with_provider<T, F, Fut>(&self, chain: Chain, call: F) -> Result<T, GatewayError>
    where
        F: FnOnce(Arc<dyn ChainTraits>) -> Fut,
        Fut: Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    {
        let provider = self.provider(chain).await?;
        call(provider).await.map_err(|e| GatewayError::NetworkError { msg: e.to_string() })
    }

    pub async fn provider(&self, chain: Chain) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        self.chain_factory.create(chain).await
    }

    pub async fn provider_with_url(&self, chain: Chain, url: String) -> Result<Arc<dyn ChainTraits>, GatewayError> {
        self.chain_factory.create_with_url(chain, url).await
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
    pub fn new(provider: Arc<dyn AlienProvider>, preferences: Arc<dyn GemPreferences>, secure_preferences: Arc<dyn GemPreferences>, api_url: String) -> Self {
        let api_client = GemApiClient::new(api_url, provider.clone());
        let chain_factory = Arc::new(ChainClientFactory::new(provider.clone(), preferences, secure_preferences));
        let yielder = Yielder::new(Arc::new(AlienProviderWrapper::new(provider)));
        Self {
            api_client,
            chain_factory,
            yielder,
        }
    }

    pub async fn get_balance_coin(&self, chain: Chain, address: String) -> Result<GemAssetBalance, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_coin(address).await }).await
    }

    pub async fn get_balance_tokens(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_tokens(address, token_ids).await })
            .await
    }

    pub async fn get_balance_staking(&self, chain: Chain, address: String) -> Result<Option<GemAssetBalance>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_balance_staking(address).await }).await
    }

    pub async fn get_staking_validators(&self, chain: Chain, apy: Option<f64>) -> Result<Vec<GemDelegationValidator>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_staking_validators(apy).await }).await
    }

    pub async fn get_staking_delegations(&self, chain: Chain, address: String) -> Result<Vec<GemDelegationBase>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_staking_delegations(address).await }).await
    }

    pub async fn transaction_broadcast(&self, chain: Chain, data: String, options: GemBroadcastOptions) -> Result<String, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.transaction_broadcast(data, options).await })
            .await
    }

    pub async fn get_transaction_status(&self, chain: Chain, request: GemTransactionStateRequest) -> Result<GemTransactionUpdate, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_transaction_status(request.into()).await })
            .await
    }

    pub async fn get_chain_id(&self, chain: Chain) -> Result<String, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_chain_id().await }).await
    }

    pub async fn get_block_number(&self, chain: Chain) -> Result<u64, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_block_latest_number().await }).await
    }

    pub async fn get_fee_rates(&self, chain: Chain, input: GemTransactionInputType) -> Result<Vec<GemFeeRate>, GatewayError> {
        let fees = self
            .with_provider(chain, |provider| async move { provider.get_transaction_fee_rates(input.into()).await })
            .await?;
        Ok(fees.into_iter().map(|f| f.into()).collect())
    }

    pub async fn get_utxos(&self, chain: Chain, address: String) -> Result<Vec<GemUTXO>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_utxos(address).await }).await
    }

    pub async fn get_address_status(&self, chain: Chain, address: String) -> Result<Vec<GemAddressStatus>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_address_status(address).await }).await
    }

    pub async fn get_transaction_preload(&self, chain: Chain, input: GemTransactionPreloadInput) -> Result<GemTransactionLoadMetadata, GatewayError> {
        let preload_input: primitives::TransactionPreloadInput = input.into();
        let metadata = self
            .with_provider(chain, |provider| async move { provider.get_transaction_preload(preload_input).await })
            .await?;
        Ok(metadata.into())
    }

    pub async fn get_transaction_scan(&self, _chain: Chain, input: GemTransactionPreloadInput) -> Result<Option<GemScanTransaction>, GatewayError> {
        let preload_input: TransactionPreloadInput = input.into();

        let Some(scan_type) = preload_input.scan_type() else {
            return Ok(None);
        };

        let payload = ScanTransactionPayload {
            origin: ScanAddressTarget {
                asset_id: preload_input.input_type.get_asset().id.clone(),
                address: preload_input.sender_address.clone(),
            },
            target: ScanAddressTarget {
                asset_id: preload_input.input_type.get_recipient_asset().id.clone(),
                address: preload_input.destination_address.clone(),
            },
            website: preload_input.get_website(),
            transaction_type: scan_type,
        };

        self.api_client.scan_transaction(payload).await.map(Some).map_err(|e| GatewayError::NetworkError { msg: e })
    }

    pub async fn get_fee(&self, chain: Chain, input: GemTransactionLoadInput, provider: Arc<dyn GemGatewayEstimateFee>) -> Result<Option<GemTransactionLoadFee>, GatewayError> {
        let fee = provider.get_fee(chain, input.clone()).await?;
        if let Some(fee) = fee {
            return Ok(Some(fee));
        }
        if let Some(fee_data) = provider.get_fee_data(chain, input.clone()).await? {
            let data = self
                .with_provider(chain, |chain_provider| async move { chain_provider.get_transaction_fee_from_data(fee_data).await })
                .await?;
            return Ok(Some(data.into()));
        }
        Ok(None)
    }

    pub async fn get_transaction_load(&self, chain: Chain, input: GemTransactionLoadInput, provider: Arc<dyn GemGatewayEstimateFee>) -> Result<GemTransactionData, GatewayError> {
        let fee = self.get_fee(chain, input.clone(), provider.clone()).await?;

        let load_data = self
            .with_provider(chain, |chain_provider| async move { chain_provider.get_transaction_load(input.clone().into()).await })
            .await?;

        let data = if let Some(fee) = fee { load_data.new_from(fee.into()) } else { load_data };

        Ok(GemTransactionData {
            fee: data.fee.into(),
            metadata: data.metadata.into(),
        })
    }

    pub async fn get_positions(&self, chain: Chain, address: String) -> Result<GemPerpetualPositionsSummary, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_positions(address).await }).await
    }

    pub async fn get_perpetuals_data(&self, chain: Chain) -> Result<Vec<GemPerpetualData>, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_perpetuals_data().await }).await
    }

    pub async fn get_perpetual_candlesticks(&self, chain: Chain, symbol: String, period: String) -> Result<Vec<GemChartCandleStick>, GatewayError> {
        let chart_period = ChartPeriod::new(period).unwrap();
        self.with_provider(chain, |provider| async move { provider.get_perpetual_candlesticks(symbol, chart_period).await })
            .await
    }

    pub async fn get_perpetual_portfolio(&self, chain: Chain, address: String) -> Result<GemPerpetualPortfolio, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_perpetual_portfolio(address).await }).await
    }

    pub async fn get_token_data(&self, chain: Chain, token_id: String) -> Result<GemAsset, GatewayError> {
        self.with_provider(chain, |provider| async move { provider.get_token_data(token_id).await }).await
    }

    pub async fn get_is_token_address(&self, chain: Chain, token_id: String) -> Result<bool, GatewayError> {
        Ok(self.provider(chain).await?.get_is_token_address(&token_id))
    }

    pub async fn get_balance_earn(&self, chain: Chain, address: String, token_ids: Vec<String>) -> Result<Vec<GemAssetBalance>, GatewayError> {
        Ok(self.yielder.get_balance(chain, &address, &token_ids).await)
    }

    pub async fn get_earn_data(&self, asset_id: AssetId, address: String, value: String, earn_type: GemEarnType) -> Result<GemContractCallData, GatewayError> {
        self.yielder
            .get_data(&asset_id, &address, &value, &earn_type)
            .await
            .map_err(|e| GatewayError::NetworkError { msg: e.to_string() })
    }

    pub fn get_earn_providers(&self, asset_id: AssetId) -> Vec<GemDelegationValidator> {
        self.yielder.get_providers(&asset_id)
    }

    pub async fn get_earn_positions(&self, address: String, asset_id: AssetId) -> Vec<GemDelegationBase> {
        self.yielder.get_positions(&address, &asset_id).await
    }

    pub async fn get_node_status(&self, chain: Chain, url: &str) -> Result<GemNodeStatus, GatewayError> {
        let start_time = std::time::Instant::now();
        let provider = self.provider_with_url(chain, url.to_string()).await?;

        let (chain_id, latest_block_number) = futures::try_join!(provider.get_chain_id(), provider.get_block_latest_number()).map_err(map_network_error)?;

        let latency_ms = start_time.elapsed().as_millis() as u64;

        Ok(GemNodeStatus {
            chain_id,
            latest_block_number,
            latency_ms,
        })
    }
}

#[cfg(all(test, feature = "reqwest_provider"))]
mod tests {
    use super::*;
    use crate::alien::reqwest_provider::NativeProvider;

    #[tokio::test]
    async fn test_get_node_status_http_404_error() {
        let provider: Arc<dyn AlienProvider> = Arc::new(NativeProvider::new().set_debug(false));
        let preferences: Arc<dyn GemPreferences> = Arc::new(EmptyPreferences {});
        let gateway = GemGateway::new(provider, preferences.clone(), preferences.clone(), "https://example.invalid".to_string());

        let result = gateway.get_node_status(Chain::Bitcoin, "https://httpbin.org/status/404").await;

        match result {
            Ok(status) => panic!("expected network error for 404 response, got {:?}", status),
            Err(GatewayError::NetworkError { msg }) => assert_eq!(msg, "HTTP error: status 404"),
            Err(GatewayError::PlatformError { .. }) => panic!("expected NetworkError, got PlatformError"),
        }
    }
}
