use super::model::RouteData;
use super::DEFAULT_DEPOSIT_GAS_LIMIT;
use super::{asset::THORChainAsset, chain::THORChainName, ThorChain, QUOTE_INTERVAL, QUOTE_MINIMUM, QUOTE_QUANTITY};
use crate::network::AlienProvider;
use crate::swapper::approval::check_approval_erc20;
use crate::swapper::asset::{
    AVALANCHE_USDC, AVALANCHE_USDT, BASE_CBBTC, BASE_USDC, ETHEREUM_DAI, ETHEREUM_USDC, ETHEREUM_USDT, ETHEREUM_WBTC, SMARTCHAIN_USDC, SMARTCHAIN_USDT,
};
use crate::swapper::thorchain::client::ThorChainSwapClient;
use crate::swapper::{ApprovalData, FetchQuoteData, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError};
use crate::swapper::{GemSwapProvider, SwapChainAsset};
use alloy_core::sol_types::SolCall;
use alloy_primitives::Address;
use alloy_primitives::U256;
use async_trait::async_trait;
use gem_evm::thorchain::contracts::RouterInterface;
use primitives::Chain;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[async_trait]
impl GemSwapProvider for ThorChain {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Thorchain
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all()
            .into_iter()
            .filter_map(|chain| THORChainName::from_chain(&chain).map(|name| name.chain()))
            .collect::<Vec<Chain>>()
            .into_iter()
            .map(|chain| match chain {
                Chain::Ethereum => SwapChainAsset::Assets(
                    chain,
                    vec![
                        ETHEREUM_USDT.id.clone(),
                        ETHEREUM_USDC.id.clone(),
                        ETHEREUM_DAI.id.clone(),
                        ETHEREUM_WBTC.id.clone(),
                    ],
                ),
                Chain::SmartChain => SwapChainAsset::Assets(chain, vec![SMARTCHAIN_USDT.id.clone(), SMARTCHAIN_USDC.id.clone()]),
                Chain::AvalancheC => SwapChainAsset::Assets(chain, vec![AVALANCHE_USDT.id.clone(), AVALANCHE_USDC.id.clone()]),
                Chain::Base => SwapChainAsset::Assets(chain, vec![BASE_USDC.id.clone(), BASE_CBBTC.id.clone()]),
                _ => SwapChainAsset::Assets(chain, vec![]),
            })
            .collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let endpoint = provider
            .get_endpoint(Chain::Thorchain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        let client = ThorChainSwapClient::new(provider.clone());

        let from_asset = THORChainAsset::from_asset_id(request.clone().from_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(request.clone().to_asset).ok_or(SwapperError::NotSupportedAsset)?;

        let value = self.value_from(request.clone().value, from_asset.decimals as i32);
        let fee = request.options.clone().fee.unwrap_or_default().thorchain;

        let quote = client
            .get_quote(
                endpoint.as_str(),
                from_asset.clone(),
                to_asset.clone(),
                value.to_string(),
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps.into(),
            )
            .await?;

        let to_value = self.value_to(quote.expected_amount_out, to_asset.decimals as i32);

        let route_data = RouteData {
            router_address: quote.router.clone(),
            inbound_address: quote.inbound_address.clone(),
        };

        let quote = SwapQuote {
            from_value: request.clone().value,
            to_value: to_value.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: serde_json::to_string(&route_data).unwrap_or_default(),
                    gas_limit: None,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
        };

        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let fee = quote.request.options.clone().fee.unwrap_or_default().thorchain;
        let from_asset = THORChainAsset::from_asset_id(quote.clone().request.from_asset).ok_or(SwapperError::NotSupportedAsset)?;
        let to_asset = THORChainAsset::from_asset_id(quote.clone().request.to_asset).ok_or(SwapperError::NotSupportedAsset)?;

        let memo = to_asset
            .get_memo(
                quote.request.destination_address.clone(),
                QUOTE_MINIMUM,
                QUOTE_INTERVAL,
                QUOTE_QUANTITY,
                fee.address,
                fee.bps,
            )
            .unwrap();

        let route_data: RouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let value = quote.request.value.clone();

        let approval: Option<ApprovalData> = {
            if from_asset.use_evm_router() {
                let from_amount: U256 = value.to_string().parse().map_err(|_| SwapperError::InvalidAmount)?;
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    from_asset.token_id.clone().unwrap(),
                    route_data.router_address.clone().unwrap(),
                    from_amount,
                    provider.clone(),
                    &from_asset.chain.chain(),
                )
                .await?
                .approval_data()
            } else {
                None
            }
        };

        let data = if from_asset.use_evm_router() {
            // only used for swapping from ERC20 tokens
            let to = route_data.router_address.clone().unwrap();
            let inbound_address = Address::from_str(&route_data.inbound_address.unwrap_or_default()).unwrap();
            let token_address = Address::from_str(&quote.request.from_asset.token_id.clone().unwrap()).unwrap();
            let amount = U256::from_str(&value).unwrap();
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 86400; // + 1 day
            let expiry = U256::from_str(timestamp.to_string().as_str()).unwrap();

            let call = RouterInterface::depositWithExpiryCall {
                inbound_address,
                token_address,
                amount,
                memo,
                expiry,
            }
            .abi_encode();

            SwapQuoteData {
                to,
                value: "0".to_string(),
                data: hex::encode(call.clone()),
                approval,
                gas_limit: Some(DEFAULT_DEPOSIT_GAS_LIMIT.to_string()),
            }
        } else {
            SwapQuoteData {
                to: route_data.inbound_address.unwrap_or_default(),
                value,
                data: self.data(from_asset.chain, memo),
                approval,
                gas_limit: None,
            }
        };

        Ok(data)
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let endpoint = provider
            .get_endpoint(Chain::Thorchain)
            .map_err(|err| SwapperError::NetworkError { msg: err.to_string() })?;
        let client = ThorChainSwapClient::new(provider);

        let status = client.get_transaction_status(&endpoint, transaction_hash).await?;

        Ok(status.observed_tx.status == "done")
    }
}
