use std::sync::Arc;

use async_trait::async_trait;
use gem_client::Client;
use primitives::{AssetId, Chain, chain_cosmos::CosmosChain};

use super::{SQUID_COSMOS_MULTICALL, SUPPORTED_CHAINS, client::SquidClient, model::*};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData, config::get_swap_api_url, cross_chain::VaultAddresses, fees::resolve_max_quote_value,
};

#[derive(Debug)]
pub struct Squid<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    provider: ProviderType,
    client: SquidClient<C>,
}

impl Squid<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = SquidClient::new(RpcClient::new(get_swap_api_url("squid"), rpc_provider));
        Self {
            provider: ProviderType::new(SwapperProvider::Squid),
            client,
        }
    }
}

fn get_network_id(chain: &Chain) -> Result<&str, SwapperError> {
    CosmosChain::from_chain(*chain).map(|_| chain.network_id()).ok_or(SwapperError::NotSupportedChain)
}

fn get_token_id(asset_id: &AssetId) -> Result<String, SwapperError> {
    if asset_id.is_native() {
        asset_id.chain.as_denom().map(|d| d.to_string()).ok_or(SwapperError::NotSupportedAsset)
    } else {
        asset_id.token_id.clone().ok_or(SwapperError::NotSupportedAsset)
    }
}

fn build_route_request(request: &QuoteRequest, from_value: &str, quote_only: bool) -> Result<SquidRouteRequest, SwapperError> {
    let from_asset_id = request.from_asset.asset_id();
    let to_asset_id = request.to_asset.asset_id();
    Ok(SquidRouteRequest {
        from_chain: get_network_id(&from_asset_id.chain)?.to_string(),
        to_chain: get_network_id(&to_asset_id.chain)?.to_string(),
        from_token: get_token_id(&from_asset_id)?,
        to_token: get_token_id(&to_asset_id)?,
        from_amount: from_value.to_string(),
        from_address: request.wallet_address.clone(),
        to_address: request.destination_address.clone(),
        slippage_config: SlippageConfig { auto_mode: 1 },
        quote_only,
    })
}

#[async_trait]
impl<C> Swapper for Squid<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        SUPPORTED_CHAINS.clone()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_value = resolve_max_quote_value(request)?;
        let squid_request = build_route_request(request, &from_value, true)?;
        let response = self.client.get_route(&squid_request).await?;

        let from_asset_id = request.from_asset.asset_id();
        let to_asset_id = request.to_asset.asset_id();

        Ok(Quote {
            from_value,
            to_value: response.route.estimate.to_amount,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset_id,
                    output: to_asset_id,
                    route_data: String::new(),
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(response.route.estimate.estimated_route_duration),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = build_route_request(&quote.request, &quote.from_value, false)?;
        let response = self.client.get_route(&request).await?;
        let tx = response.route.transaction_request.ok_or(SwapperError::InvalidRoute)?;

        Ok(SwapperQuoteData::new_contract(tx.target, tx.value, tx.data, None, Some(tx.gas_limit)))
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let address = SQUID_COSMOS_MULTICALL.to_string();
        Ok(VaultAddresses {
            deposit: vec![address.clone()],
            send: vec![address],
        })
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let result = self.client.get_status(transaction_hash).await?;
        Ok(SwapResult {
            status: result.squid_transaction_status.swap_status(),
            metadata: None,
        })
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperMode, SwapperQuoteAsset, models::Options};
    use primitives::swap::SwapStatus;

    const OSMOSIS_ADDRESS: &str = "osmo1tkvyjqeq204rmrrz3w4hcrs336qahsfwn8m0ye";
    const COSMOS_ADDRESS: &str = "cosmos1tkvyjqeq204rmrrz3w4hcrs336qahsfwmugljt";

    fn create_provider() -> Squid<RpcClient> {
        let provider = Arc::new(crate::NativeProvider::default());
        Squid::new(provider)
    }

    #[tokio::test]
    async fn test_squid_osmo_to_atom() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let squid = create_provider();

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Osmosis)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Cosmos)),
            wallet_address: OSMOSIS_ADDRESS.to_string(),
            destination_address: COSMOS_ADDRESS.to_string(),
            value: "10000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options::new_with_slippage(100.into()),
        };

        let quote = squid.get_quote(&request).await?;
        println!(
            "OSMO->ATOM quote: from={}, to={}, eta={}s",
            quote.from_value,
            quote.to_value,
            quote.eta_in_seconds.unwrap_or(0)
        );
        assert_eq!(quote.from_value, "10000000");
        assert!(!quote.to_value.is_empty());
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = squid.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("OSMO->ATOM data: to={}, value={}, gasLimit={:?}", quote_data.to, quote_data.value, quote_data.gas_limit);
        println!("OSMO->ATOM msg: {}", &quote_data.data[..200.min(quote_data.data.len())]);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_squid_atom_to_osmo() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let squid = create_provider();

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Cosmos)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Osmosis)),
            wallet_address: COSMOS_ADDRESS.to_string(),
            destination_address: OSMOSIS_ADDRESS.to_string(),
            value: "1000000".to_string(),
            mode: SwapperMode::ExactIn,
            options: Options::new_with_slippage(100.into()),
        };

        let quote = squid.get_quote(&request).await?;
        println!(
            "ATOM->OSMO quote: from={}, to={}, eta={}s",
            quote.from_value,
            quote.to_value,
            quote.eta_in_seconds.unwrap_or(0)
        );
        assert_eq!(quote.from_value, "1000000");
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = squid.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("ATOM->OSMO data: to={}, value={}, gasLimit={:?}", quote_data.to, quote_data.value, quote_data.gas_limit);
        println!("ATOM->OSMO msg: {}", &quote_data.data[..200.min(quote_data.data.len())]);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_squid_swap_status() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let squid = create_provider();
        let result = squid
            .get_swap_result(Chain::Cosmos, "D68723CEADAB65795B176FAE0B84B0ED5923DA9AAEC69502F8D30554431250A9")
            .await?;
        println!("status: {:?}", result.status);
        assert_eq!(result.status, SwapStatus::Completed);
        Ok(())
    }
}
