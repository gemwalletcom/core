use std::sync::Arc;

use async_trait::async_trait;
use gem_client::Client;
use gem_cosmos::{converter::convert_cosmos_address, models::message::send_msg_json};
use primitives::{AssetId, Chain, chain_cosmos::CosmosChain, swap::SwapQuoteDataType};

use super::{SQUID_COSMOS_MULTICALL, SUPPORTED_CHAINS, client::SquidClient, model::*};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    config::{DEFAULT_SWAP_FEE_BPS, get_swap_api_url},
    cross_chain::VaultAddresses,
    fees::resolve_max_quote_value,
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

impl<C> Squid<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
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

    fn get_fee_address(request: &QuoteRequest) -> Option<String> {
        let fees = request.options.fee.as_ref()?;
        let chain = request.from_asset.chain();
        match chain {
            Chain::Injective => Some(fees.injective.address.clone()).filter(|a| !a.is_empty()),
            Chain::Cosmos => Some(fees.cosmos.address.clone()).filter(|a| !a.is_empty()),
            _ => {
                let cosmos_chain = CosmosChain::from_chain(chain)?;
                let base = &fees.cosmos.address;
                if base.is_empty() {
                    return None;
                }
                convert_cosmos_address(base, cosmos_chain.hrp()).ok()
            }
        }
    }

    fn build_route_request(request: &QuoteRequest, from_value: &str, quote_only: bool) -> Result<SquidRouteRequest, SwapperError> {
        let from_asset_id = request.from_asset.asset_id();
        let to_asset_id = request.to_asset.asset_id();
        Ok(SquidRouteRequest {
            from_chain: Self::get_network_id(&from_asset_id.chain)?.to_string(),
            to_chain: Self::get_network_id(&to_asset_id.chain)?.to_string(),
            from_token: Self::get_token_id(&from_asset_id)?,
            to_token: Self::get_token_id(&to_asset_id)?,
            from_amount: from_value.to_string(),
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            slippage_config: SlippageConfig { auto_mode: 1 },
            quote_only,
        })
    }
}

impl<C> Squid<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    async fn fetch_route(&self, request: &QuoteRequest, from_value: &str, quote_only: bool) -> Result<(SquidRouteResponse, u128, Option<String>), SwapperError> {
        let fee_address = Self::get_fee_address(request);
        let value: u128 = from_value.parse().unwrap_or(0);
        let fee = if fee_address.is_some() { value * DEFAULT_SWAP_FEE_BPS as u128 / 10_000 } else { 0 };
        let swap_value = (value - fee).to_string();
        let squid_request = Self::build_route_request(request, &swap_value, quote_only)?;
        let response = self.client.get_route(&squid_request).await?;
        Ok((response, fee, fee_address))
    }
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
        let (response, _, _) = self.fetch_route(request, &from_value, true).await?;

        Ok(Quote {
            from_value,
            to_value: response.route.estimate.to_amount,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: String::new(),
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(response.route.estimate.estimated_route_duration),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let (response, fee, fee_address) = self.fetch_route(&quote.request, &quote.from_value, false).await?;
        let tx = response.route.transaction_request.ok_or(SwapperError::InvalidRoute)?;

        let swap_msg: serde_json::Value = serde_json::from_str(&tx.data).map_err(|e| SwapperError::TransactionError(e.to_string()))?;
        let messages = match fee_address {
            Some(addr) if fee > 0 => {
                let denom = Self::get_token_id(&quote.request.from_asset.asset_id())?;
                vec![send_msg_json(&quote.request.wallet_address, &addr, &denom, &fee.to_string()), swap_msg]
            }
            _ => vec![swap_msg],
        };
        let data = serde_json::to_string(&messages).map_err(|e| SwapperError::TransactionError(e.to_string()))?;

        Ok(SwapperQuoteData {
            to: tx.target,
            data_type: SwapQuoteDataType::Contract,
            value: tx.value,
            data,
            memo: None,
            approval: None,
            gas_limit: Some(tx.gas_limit),
        })
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
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);

        let quote_data = squid.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("OSMO->ATOM data: to={}, value={}, gasLimit={:?}", quote_data.to, quote_data.value, quote_data.gas_limit);
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
