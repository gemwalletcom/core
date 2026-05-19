use crate::{
    FetchQuoteData, Permit2ApprovalData, ProviderData, ProviderType, Quote, QuoteRequest, Swapper, SwapperError, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::{check_approval_erc20_with_client, check_approval_permit2_with_client, get_swap_gas_limit_with_approval},
    eth_address,
    fees::{apply_slippage_in_bp, quote_value_after_reserve_by_chain},
    models::*,
    uniswap::{
        deadline::get_sig_deadline,
        fee_token::{FeeToken, get_fee_token},
        quote_result::{get_best_quote, get_selected_candidate},
        requires_native_wrapping,
        swap_route::{RouteData, build_swap_route},
    },
};
use alloy_primitives::{Address, Bytes, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{command::encode_commands, path::get_base_pair},
};
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};
use std::{fmt, str::FromStr, sync::Arc};

use super::{DEFAULT_SWAP_GAS_LIMIT, UniversalRouterProvider, commands::build_commands, path::build_paths_with_routes};

pub struct UniswapV3 {
    provider: Box<dyn UniversalRouterProvider>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl UniswapV3 {
    pub fn new(provider: Box<dyn UniversalRouterProvider>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self { provider, rpc_provider }
    }

    pub fn support_chain(&self, chain: &Chain) -> bool {
        self.provider.get_deployment_by_chain(chain).is_some()
    }

    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<RpcClient>, SwapperError> {
        let endpoint = self.rpc_provider.get_endpoint(chain).map_err(SwapperError::from)?;
        let client = RpcClient::new(endpoint, self.rpc_provider.clone());
        Ok(JsonRpcClient::new(client))
    }

    fn get_asset_address(asset_id: &str, evm_chain: EVMChain) -> Result<Address, SwapperError> {
        let asset_id = AssetId::new(asset_id).ok_or(SwapperError::NotSupportedAsset)?;
        eth_address::parse_or_weth_address(&asset_id, evm_chain)
    }

    fn parse_request_value(request: &QuoteRequest, value: &str) -> Result<(EVMChain, Address, Address, U256), SwapperError> {
        let evm_chain = EVMChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let token_in = Self::get_asset_address(&request.from_asset.id, evm_chain)?;
        let token_out = Self::get_asset_address(&request.to_asset.id, evm_chain)?;
        let amount_in = U256::from_str(value).map_err(SwapperError::from)?;

        Ok((evm_chain, token_in, token_out, amount_in))
    }

    async fn check_erc20_approval(
        &self,
        client: &JsonRpcClient<RpcClient>,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
    ) -> Result<ApprovalType, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let spender = deployment.permit2.to_string();
        check_approval_erc20_with_client(wallet_address.to_string(), token.to_string(), spender, amount, client).await
    }

    async fn check_permit2_approval(
        &self,
        client: &JsonRpcClient<RpcClient>,
        wallet_address: Address,
        token: &str,
        amount: U256,
        chain: &Chain,
    ) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(chain).ok_or(SwapperError::NotSupportedChain)?;

        Ok(check_approval_permit2_with_client(
            deployment.permit2,
            wallet_address.to_string(),
            token.to_string(),
            deployment.universal_router.to_string(),
            amount,
            client,
        )
        .await?
        .permit2_data())
    }
}

impl fmt::Debug for UniswapV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV3").finish()
    }
}

#[async_trait]
impl Swapper for UniswapV3 {
    fn provider(&self) -> &ProviderType {
        self.provider.provider()
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapperChainAsset::All(*x)).collect()
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let from_value = quote_value_after_reserve_by_chain(request)?;
        let (evm_chain, token_in, token_out, from_value) = Self::parse_request_value(request, &from_value)?;
        if requires_native_wrapping(&request.from_asset.asset_id()) || requires_native_wrapping(&request.to_asset.asset_id()) {
            _ = evm_chain.weth_contract().ok_or(SwapperError::NotSupportedChain)?;
        }

        let client = Arc::new(self.client_for(from_chain)?);

        let fee_tiers = self.provider.get_tiers();
        let use_weth = evm_chain.weth_contract().is_some();
        let base_pair = get_base_pair(&evm_chain, use_weth).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;

        let fee_token_in = FeeToken::new(token_in, request.from_asset.symbol.as_str());
        let fee_token_out = FeeToken::new(token_out, request.to_asset.symbol.as_str());
        let fee_preference = get_fee_token(Some(&base_pair), &fee_token_in, &fee_token_out);
        let fee_bps = request.options.fee.as_ref().map_or(0, |fees| fees.evm.bps);

        let quote_amount_in = if fee_preference.is_input_token && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
        };

        let paths_array = super::path::build_paths(&token_in, &token_out, &fee_tiers, &base_pair);
        let requests: Vec<_> = paths_array
            .iter()
            .map(|paths| {
                let client = Arc::clone(&client);
                let calls: Vec<EthereumRpc> = paths
                    .iter()
                    .map(|path| super::quoter_v2::build_quoter_request(&request.wallet_address, deployment.quoter_v2, quote_amount_in, &path.1))
                    .collect();
                async move { client.batch_call_requests(calls).await }
            })
            .collect();

        let batch_results = futures::future::join_all(requests).await;

        let quote_result = get_best_quote(&batch_results, super::quoter_v2::decode_quoter_response)?;
        let selected_path = get_selected_candidate(&paths_array, &quote_result)?;

        let to_value = if fee_preference.is_input_token {
            quote_result.amount_out
        } else {
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let to_min_value = apply_slippage_in_bp(&to_value, request.options.slippage.bps);

        let routes = build_swap_route(from_chain, &selected_path.0, &to_min_value.to_string())?;

        Ok(Quote {
            from_value: from_value.to_string(),
            to_value: to_value.to_string(),
            data: ProviderData {
                provider: self.provider().clone(),
                routes,
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        if requires_native_wrapping(&from_asset) {
            return Ok(None);
        }
        let client = self.client_for(from_asset.chain)?;
        let wallet_address = eth_address::parse_str(&quote.request.wallet_address)?;
        let (_, token_in, _, amount_in) = Self::parse_request_value(&quote.request, &quote.from_value)?;
        self.check_permit2_approval(&client, wallet_address, &token_in.to_checksum(None), amount_in, &from_asset.chain)
            .await
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_chain = request.from_asset.chain();
        let (_, token_in, token_out, amount_in) = Self::parse_request_value(request, &quote.from_value)?;
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = self.client_for(from_chain)?;

        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: RouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = U256::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let wallet_address = eth_address::parse_str(&request.wallet_address)?;
        let permit = data.permit2_data().map(|data| data.into());
        let wrap_input_eth = requires_native_wrapping(&request.from_asset.asset_id());

        let approval: Option<ApprovalData> = if wrap_input_eth {
            None
        } else {
            self.check_erc20_approval(&client, wallet_address, &token_in.to_checksum(None), amount_in, &from_chain)
                .await?
                .approval_data()
        };
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_SWAP_GAS_LIMIT);

        let sig_deadline = get_sig_deadline();

        let evm_chain = EVMChain::from_chain(from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let use_weth = evm_chain.weth_contract().is_some();
        let base_pair = get_base_pair(&evm_chain, use_weth);
        let fee_token_in = FeeToken::new(token_in, request.from_asset.symbol.as_str());
        let fee_token_out = FeeToken::new(token_out, request.to_asset.symbol.as_str());
        let fee_preference = get_fee_token(base_pair.as_ref(), &fee_token_in, &fee_token_out);

        let path: Bytes = build_paths_with_routes(&quote.data.routes)?;
        let commands = build_commands(request, &token_in, &token_out, amount_in, to_amount, &path, permit, fee_preference.is_input_token)?;
        let encoded = encode_commands(&commands, U256::from(sig_deadline));

        let value = if wrap_input_eth { quote.from_value.clone() } else { String::from("0") };

        Ok(SwapperQuoteData::new_contract(
            deployment.universal_router.into(),
            value,
            HexEncode(encoded),
            approval,
            gas_limit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Options, Swapper,
        alien::mock::ProviderMock,
        uniswap::{default::new_uniswap_v3, swap_route::build_swap_route},
    };
    use gem_evm::uniswap::{FeeTier, path::TokenPair};
    use primitives::asset_constants::{ETHEREUM_USDC_TOKEN_ID, ETHEREUM_WETH_TOKEN_ID};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_quote_data_uses_quote_from_value_for_native_value() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = new_uniswap_v3(provider);
        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Ethereum).into(),
            to_asset: AssetId::from(Chain::Ethereum, Some(ETHEREUM_USDC_TOKEN_ID.into())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: "2000000000000000".into(),
            options: Options {
                use_max_amount: true,
                ..Options::default()
            },
        };
        let token_pairs = vec![TokenPair {
            token_in: eth_address::parse_str(ETHEREUM_WETH_TOKEN_ID).unwrap(),
            token_out: eth_address::parse_str(ETHEREUM_USDC_TOKEN_ID).unwrap(),
            fee_tier: FeeTier::FiveHundred,
        }];
        let routes = build_swap_route(Chain::Ethereum, &token_pairs, "100").unwrap();
        let quote = Quote {
            from_value: "1000000000000000".into(),
            to_value: "100".into(),
            data: ProviderData {
                provider: swapper.provider().clone(),
                routes,
                slippage_bps: request.options.slippage.bps,
            },
            request,
            eta_in_seconds: None,
        };

        let quote_data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.value, quote.from_value);
    }
}
