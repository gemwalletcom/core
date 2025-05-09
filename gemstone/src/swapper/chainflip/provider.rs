use alloy_primitives::U256;
use num_bigint::BigUint;
use std::sync::Arc;

use super::{
    broker::{model::*, BrokerClient},
    capitalize::capitalize_first_letter,
    client::{ChainflipClient, QuoteRequest, QuoteResponse},
    price::{apply_slippage, price_to_hex_price},
    ChainflipRouteData,
};
use crate::{
    config::swap_config::DEFAULT_CHAINFLIP_FEE_BPS,
    network::AlienProvider,
    swapper::{
        approval::check_approval_erc20,
        asset::{ARBITRUM_USDC, ETHEREUM_USDC, ETHEREUM_USDT, SOLANA_USDC},
        FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProviderData, SwapProviderType, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, Swapper,
        SwapperError,
    },
};
use primitives::{chain::Chain, swap::QuoteAsset, ChainType};

#[derive(Debug)]
pub struct ChainflipProvider {
    provider: SwapProviderType,
}

impl Default for ChainflipProvider {
    fn default() -> Self {
        Self {
            provider: SwapProviderType::new(GemSwapProvider::Chainflip),
        }
    }
}

impl ChainflipProvider {
    fn map_asset_id(asset: &QuoteAsset) -> ChainflipAsset {
        let asset_id = asset.asset_id();
        let chain_name = capitalize_first_letter(asset_id.chain.as_ref());
        ChainflipAsset {
            chain: chain_name,
            asset: asset.symbol.clone(),
        }
    }

    fn get_boost_quote(&self, quote_response: &QuoteResponse) -> (String, u32, u32, Option<u32>, String) {
        let egress_amount: String;
        let slippage_bps: u32;
        let eta_in_seconds: u32;
        let boost_fee: Option<u32>;
        let estimated_price: String;

        // Use boost quote if available
        if let Some(boost_quote) = &quote_response.boost_quote {
            egress_amount = boost_quote.egress_amount.clone();
            slippage_bps = boost_quote.slippage_bps();
            eta_in_seconds = boost_quote.estimated_duration_seconds as u32;
            boost_fee = Some(boost_quote.estimated_boost_fee_bps);
            estimated_price = boost_quote.estimated_price.clone();
        } else {
            egress_amount = quote_response.egress_amount.clone();
            slippage_bps = quote_response.slippage_bps();
            eta_in_seconds = quote_response.estimated_duration_seconds as u32;
            boost_fee = None;
            estimated_price = quote_response.estimated_price.clone();
        }

        (egress_amount, slippage_bps, eta_in_seconds, boost_fee, estimated_price)
    }
}

#[async_trait::async_trait]
impl Swapper for ChainflipProvider {
    fn provider(&self) -> &SwapProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        vec![
            SwapChainAsset::Assets(Chain::Bitcoin, vec![]),
            SwapChainAsset::Assets(Chain::Ethereum, vec![ETHEREUM_USDC.id.clone(), ETHEREUM_USDT.id.clone()]),
            SwapChainAsset::Assets(Chain::Solana, vec![SOLANA_USDC.id.clone()]),
            SwapChainAsset::Assets(Chain::Arbitrum, vec![ARBITRUM_USDC.id.clone()]),
        ]
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        let chainflip_client = ChainflipClient::new(provider.clone());
        let broker_client = BrokerClient::new(provider);
        let src_asset = Self::map_asset_id(&request.from_asset);
        let dest_asset = Self::map_asset_id(&request.to_asset);
        let fee_bps = DEFAULT_CHAINFLIP_FEE_BPS;
        let amount = request.value.parse::<U256>()?;

        let quote_request = QuoteRequest {
            amount: request.value.clone(),
            src_chain: src_asset.chain.clone(),
            src_asset: src_asset.asset.clone(),
            dest_chain: dest_asset.chain,
            dest_asset: dest_asset.asset,
            is_vault_swap: true,
            dca_enabled: true,
            broker_commission_bps: Some(fee_bps),
        };

        let swap_limit_req = broker_client.get_swap_limits();
        let quote_req = chainflip_client.get_quote(&quote_request);
        let (swap_limit, quote_responses) = futures::try_join!(swap_limit_req, quote_req)?;

        if swap_limit.get_min_deposit_amount(&src_asset)? > amount {
            return Err(SwapperError::InputAmountTooSmall);
        }

        if quote_responses.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let quote_response = &quote_responses[0];
        let (egress_amount, slippage_bps, eta_in_seconds, boost_fee, estimated_price) = self.get_boost_quote(quote_response);
        let route_data = ChainflipRouteData {
            boost_fee,
            fee_bps,
            estimated_price,
        };
        let quote = SwapQuote {
            from_value: request.value.clone(),
            to_value: egress_amount,
            data: SwapProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![SwapRoute {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).unwrap(),
                    gas_limit: None,
                }],
            },
            eta_in_seconds: Some(eta_in_seconds),
            request: request.clone(),
        };
        Ok(quote)
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, provider: Arc<dyn AlienProvider>, _data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let broker_client = BrokerClient::new(provider.clone());
        let source_asset = Self::map_asset_id(&quote.request.from_asset);
        let destination_asset = Self::map_asset_id(&quote.request.to_asset);

        let input_amount: BigUint = quote.request.value.parse()?;

        let route_data: ChainflipRouteData = serde_json::from_str(&quote.data.routes[0].route_data)?;
        let price = route_data
            .estimated_price
            .parse::<f64>()
            .map_err(|_| SwapperError::TransactionError("Invalid price".to_string()))?;
        let price_slippage = apply_slippage(price, quote.data.slippage_bps, true);
        let quote_asset_decimals = quote.request.to_asset.decimals;
        let base_asset_decimals = quote.request.from_asset.decimals;
        let min_price = price_to_hex_price(price_slippage, quote_asset_decimals, base_asset_decimals).map_err(SwapperError::TransactionError)?;

        let extra_params = VaultSwapEvmExtras {
            chain: source_asset.chain.clone(),
            input_amount: input_amount.clone(),
            refund_parameters: RefundParameters {
                retry_duration: 150, // blocks
                refund_address: quote.request.wallet_address.clone(),
                min_price,
            },
        };

        let response = broker_client
            .encode_vault_swap(
                source_asset,
                destination_asset,
                quote.request.destination_address.clone(),
                route_data.fee_bps,
                route_data.boost_fee,
                Some(extra_params),
                None,
            )
            .await?;

        let approval: Option<_> = if from_asset.chain.chain_type() == ChainType::Ethereum && !from_asset.is_native() {
            let approval = check_approval_erc20(
                quote.request.wallet_address.clone(),
                from_asset.token_id.unwrap(),
                response.to.clone(),
                U256::from_le_slice(&input_amount.to_bytes_le()),
                provider.clone(),
                &from_asset.chain,
            )
            .await?;
            approval.approval_data()
        } else {
            None
        };

        let swap_quote_data = SwapQuoteData {
            to: response.to,
            value: quote.request.value.clone(),
            data: response.calldata,
            approval,
            gas_limit: None,
        };

        Ok(swap_quote_data)
    }

    async fn get_transaction_status(&self, _chain: Chain, transaction_hash: &str, provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let chainflip_client = ChainflipClient::new(provider.clone());
        let status = chainflip_client.get_tx_status(transaction_hash).await?;
        Ok(status.state == "COMPLETED")
    }
}
