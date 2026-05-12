use super::{client::PanoraClient, model};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset,
    SwapperQuoteData,
    config::get_swap_proxy_url,
    fees::{ReferralFee, bps_to_percent_string, default_referral_fees, quote_value_after_reserve_by_chain},
};
use async_trait::async_trait;
use gem_aptos::{APTOS_NATIVE_COIN, ENTRY_FUNCTION_PAYLOAD_TYPE, TransactionPayload};
use gem_client::Client;
use number_formatter::BigNumberFormatter;
use primitives::Chain;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct Panora<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: PanoraClient<C>,
}

impl Panora<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(get_swap_proxy_url("panora"), rpc_provider))
    }
}

impl<C> Panora<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Panora),
            client: PanoraClient::new(client),
        }
    }

    fn referral_fee(request: &QuoteRequest) -> ReferralFee {
        request.options.fee.clone().map(|fees| fees.aptos).unwrap_or_else(|| default_referral_fees().aptos)
    }

    fn build_request(request: &QuoteRequest, from_value: &str) -> Result<model::QuoteRequest, SwapperError> {
        let referral = Self::referral_fee(request);
        Ok(model::QuoteRequest {
            from_token_address: token_address(&request.from_asset),
            to_token_address: token_address(&request.to_asset),
            from_token_amount: BigNumberFormatter::value(from_value, request.from_asset.decimals as i32)?,
            to_wallet_address: request.destination_address.clone(),
            slippage_percentage: bps_to_percent_string(request.options.slippage.bps)?,
            integrator_fee_percentage: bps_to_percent_string(referral.bps)?,
            integrator_fee_address: referral.address,
        })
    }
}

#[async_trait]
impl<C> Swapper for Panora<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Aptos)]
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let response = self.client.get_quote(&Self::build_request(request, &from_value)?).await?;
        let quote = response.quotes.first().ok_or(SwapperError::NoQuoteAvailable)?;

        Ok(Quote {
            from_value,
            to_value: BigNumberFormatter::value_from_amount(&quote.to_token_amount, response.to_token.decimals)?,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&response)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let response: model::QuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let transaction = response.quotes.first().ok_or(SwapperError::InvalidRoute)?.transaction_data.clone();

        let payload = TransactionPayload {
            function: Some(transaction.function),
            type_arguments: transaction.type_arguments,
            arguments: transaction.arguments,
            payload_type: ENTRY_FUNCTION_PAYLOAD_TYPE.to_string(),
        };

        Ok(SwapperQuoteData::new_contract(String::new(), "0".to_string(), serde_json::to_string(&payload)?, None, None))
    }
}

fn token_address(asset: &SwapperQuoteAsset) -> String {
    asset.asset_id().token_id.unwrap_or_else(|| APTOS_NATIVE_COIN.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset, fees::default_referral_fees};
    use primitives::asset_constants::APTOS_USDT_ASSET_ID;

    const TEST_WALLET: &str = "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef";
    const QUOTE_RESPONSE: &str = include_str!("testdata/quote_response.json");

    fn quote_request() -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset {
                id: Chain::Aptos.as_ref().to_string(),
                symbol: "APT".to_string(),
                decimals: 8,
            },
            to_asset: SwapperQuoteAsset {
                id: APTOS_USDT_ASSET_ID.to_string(),
                symbol: "USDT".to_string(),
                decimals: 6,
            },
            wallet_address: TEST_WALLET.to_string(),
            destination_address: TEST_WALLET.to_string(),
            value: "100000000".to_string(),
            options: Options {
                slippage: 100.into(),
                fee: Some(default_referral_fees()),
                use_max_amount: false,
            },
        }
    }

    #[test]
    fn test_build_request() {
        let request = quote_request();
        let referral = default_referral_fees().aptos;

        let built = Panora::<RpcClient>::build_request(&request, "80000000").unwrap();

        assert_eq!(built.from_token_address, APTOS_NATIVE_COIN);
        assert_eq!(built.to_token_address, token_address(&request.to_asset));
        assert_eq!(built.from_token_amount, "0.8");
        assert_eq!(built.to_wallet_address, TEST_WALLET);
        assert_eq!(built.slippage_percentage, "1");
        assert_eq!(built.integrator_fee_percentage, "0.5");
        assert_eq!(built.integrator_fee_address, referral.address);
    }

    #[test]
    fn test_parse_quote_response() {
        let response: model::QuoteResponse = serde_json::from_str(QUOTE_RESPONSE).unwrap();
        let entry = response.quotes.first().unwrap();

        assert_eq!(response.to_token.decimals, 6);
        assert_eq!(entry.to_token_amount, "0.891234");
        assert_eq!(
            entry.transaction_data.function,
            "0x1c3206329806286fd2223647c9f9b130e66baeb6d7224a18c1f642ffe48f3b4c::panora_swap::router_entry"
        );
        assert_eq!(entry.transaction_data.type_arguments, vec![APTOS_NATIVE_COIN]);
        assert_eq!(entry.transaction_data.arguments.len(), 20);
        assert_eq!(entry.transaction_data.arguments[2], serde_json::json!(1));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider};
    use primitives::{AssetId, asset_constants::APTOS_USDT_ASSET_ID};

    const TEST_APTOS_WALLET: &str = "0x4eb20e735591a85bb58921ef2e6b55c385bba10e817ffe1e02e50deb6c594aef";

    #[tokio::test]
    async fn test_panora_fetch_quote_and_quote_data() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Panora::new(rpc_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Aptos)),
            to_asset: SwapperQuoteAsset::from(APTOS_USDT_ASSET_ID.clone()),
            wallet_address: TEST_APTOS_WALLET.to_string(),
            destination_address: TEST_APTOS_WALLET.to_string(),
            value: "100000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value.parse::<u64>().unwrap() > 0);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.to, "");
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());

        Ok(())
    }
}
