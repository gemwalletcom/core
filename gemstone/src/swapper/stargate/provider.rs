use std::str::FromStr;
use std::sync::Arc;

use alloy_core::sol_types::SolCall;
use alloy_primitives::{hex, Address, Bytes, FixedBytes, U160, U256};
use async_trait::async_trait;
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    stargate::contract::{IStargate, MessagingFee, SendParam},
};
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};

use crate::{
    debug_println,
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{
        approval::check_approval_erc20, eth_rpc, slippage::apply_slippage_in_bp, ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider,
        SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute, SwapperError,
    },
};

use super::{
    endpoint::{self, StargateEndpoint, STARGATE_ROUTES},
    layer_zero::scan::LayerZeroScanApi,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StargateRouteData {
    send_param: SendParam,
    fee: MessagingFee,
    refund_address: String,
}

#[derive(Debug, Default)]
pub struct Stargate {
    pub enpoints: Vec<StargateEndpoint>,
}

impl Stargate {
    pub fn new() -> Self {
        Self {
            enpoints: vec![
                STARGATE_ROUTES.ethereum.clone(),
                STARGATE_ROUTES.base.clone(),
                STARGATE_ROUTES.optimism.clone(),
                STARGATE_ROUTES.arbitrum.clone(),
                STARGATE_ROUTES.polygon.clone(),
                STARGATE_ROUTES.avalanche.clone(),
                STARGATE_ROUTES.linea.clone(),
                STARGATE_ROUTES.smartchain.clone(),
                STARGATE_ROUTES.sei.clone(),
                STARGATE_ROUTES.mantle.clone(),
            ],
        }
    }

    pub fn get_endpoint_id(&self, chain: &Chain) -> Result<u32, SwapperError> {
        let endpoint = self.enpoints.iter().find(|x| x.id == *chain).ok_or(SwapperError::NotSupportedChain)?;
        Ok(endpoint.endpoint_id)
    }

    pub fn address_to_bytes32(&self, addr: &str) -> FixedBytes<32> {
        FixedBytes::<32>::from(U256::from(U160::from_str(addr).unwrap()))
    }

    pub fn get_pool(&self, asset: &AssetId) -> Result<String, SwapperError> {
        let endpoint = self.enpoints.iter().find(|x| x.id == asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        endpoint
            .pools
            .iter()
            .find(|x| x.asset == *asset)
            .map(|x| x.address.clone())
            .ok_or(SwapperError::NotSupportedChain)
    }
}

#[async_trait]
impl GemSwapProvider for Stargate {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Stargate
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        let mut assets = vec![];
        for endpoint in self.enpoints.iter() {
            assets.push(endpoint.chain_asset.clone());
        }

        println!("assets: {:?}", assets);
        assets
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        println!("request: {:?}", request);
        let from_asset = &request.from_asset;
        let to_asset = &request.to_asset;

        if from_asset.chain == to_asset.chain {
            return Err(SwapperError::NotSupportedChain);
        }

        if from_asset.is_native() && !to_asset.is_native() {
            return Err(SwapperError::NotSupportedChain);
        }

        let pool = self.get_pool(from_asset).unwrap();
        let amount_ld = U256::from_str(request.value.as_str()).unwrap();
        let mut send_param = SendParam {
            dstEid: self.get_endpoint_id(&to_asset.chain).unwrap(),
            to: self.address_to_bytes32(request.destination_address.as_str()),
            amountLD: amount_ld,
            minAmountLD: amount_ld,
            extraOptions: Bytes::from_str("0x").unwrap(),
            composeMsg: Bytes::from_str("0x").unwrap(),
            oftCmd: Bytes::from_str("0x").unwrap(),
        };

        println!("pool: {:?}", pool);

        println!("send_param: {:?}", send_param);

        // Encode call data
        let call_data = IStargate::quoteOFTCall {
            _sendParam: send_param.clone(),
        }
        .abi_encode();

        let call = EthereumRpc::Call(TransactionObject::new_call(pool.as_str(), call_data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, provider.clone(), &request.from_asset.chain).await?;
        let result = response.take()?;
        let hex_data = hex::decode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        let quote_oft_data = IStargate::quoteOFTCall::abi_decode_returns(&hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;

        println!("quote oft - {:?}", quote_oft_data);
        //println!("feeAmount = {}", quote_oft_data.oftFeeDetails[0].feeAmountLD);
        send_param.minAmountLD = apply_slippage_in_bp(&quote_oft_data.receipt.amountReceivedLD, request.options.slippage_bps);
        //send_param.minAmountLD = U256::from(99500u32);

        let messaging_fee_calldata = IStargate::quoteSendCall {
            _sendParam: send_param.clone(),
            _payInLzToken: false,
        }
        .abi_encode();

        let messaging_fee_call = EthereumRpc::Call(TransactionObject::new_call(pool.as_str(), messaging_fee_calldata), BlockParameter::Latest);
        let messaging_fee_response: JsonRpcResult<String> = jsonrpc_call(&messaging_fee_call, provider.clone(), &request.from_asset.chain).await?;
        let messaging_fee_result = messaging_fee_response.take()?;
        let messaging_fee_hex_data = hex::decode(messaging_fee_result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        println!("messagingFee eth_call result: {:?}", messaging_fee_hex_data);

        let messaging_fee_value =
            IStargate::quoteSendCall::abi_decode_returns(&messaging_fee_hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
        println!("messagingFee = {:?}", messaging_fee_value);

        let approval = if request.from_asset.is_token() {
            check_approval_erc20(
                request.wallet_address.clone(),
                request.from_asset.token_id.clone().unwrap(),
                pool.clone(),
                amount_ld,
                provider.clone(),
                &request.from_asset.chain,
            )
            .await?
        } else {
            ApprovalType::None
        };

        let route_data = StargateRouteData {
            send_param: send_param.clone(),
            fee: messaging_fee_value.fee,
            refund_address: request.wallet_address.to_string(),
        };

        println!("route_data: {:?}", route_data);

        Ok(SwapQuote {
            from_value: request.value.clone(),
            to_value: quote_oft_data.receipt.amountReceivedLD.to_string(),
            data: SwapProviderData {
                provider: self.provider(),
                routes: vec![SwapRoute {
                    input: request.from_asset.clone(),
                    output: request.to_asset.clone(),
                    route_data: serde_json::to_string(&route_data).unwrap_or_default(),
                    gas_estimate: None,
                }],
                suggested_slippage_bps: None,
            },
            approval,
            request: request.clone(),
        })
    }

    async fn fetch_quote_data(&self, quote: &SwapQuote, _provider: Arc<dyn AlienProvider>, data: FetchQuoteData) -> Result<SwapQuoteData, SwapperError> {
        let pool = self.get_pool(&quote.request.from_asset).unwrap();
        let route_data: StargateRouteData = serde_json::from_str(&quote.data.routes.first().unwrap().route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let send_calldata = IStargate::sendCall {
            _sendParam: route_data.send_param.clone(),
            _fee: route_data.fee.clone(),
            _refundAddress: Address::from_str(route_data.refund_address.as_str()).unwrap(),
        }
        .abi_encode();

        println!("Route data - {:?}", route_data);
        println!("Calldata - {:?}", send_calldata);
        println!("data - {:?}", data);
        let mut value_to_send = route_data.fee.nativeFee;

        if quote.request.from_asset.is_native() {
            value_to_send += route_data.send_param.amountLD;
        }

        let quote_data = SwapQuoteData {
            to: pool,
            value: value_to_send.to_string(),
            data: hex::encode_prefixed(send_calldata.clone()),
        };
        println!("Quote data - {:?}", quote_data);

        let hex_value = format!("{:#x}", value_to_send);
        println!("hex_value = {:?}", hex_value);

        let tx = TransactionObject::new_call_with_from_value(&quote.request.wallet_address, &quote_data.to, &hex_value, send_calldata);
        println!("tx = {:?}", tx);
        let gas_limit = eth_rpc::estimate_gas(_provider.clone(), &quote.request.from_asset.chain, tx).await?;
        debug_println!("gas_limit: {:?}", gas_limit);

        Ok(quote_data)
    }

    async fn get_transaction_status(&self, _chain: Chain, _transaction_hash: &str, _provider: Arc<dyn AlienProvider>) -> Result<bool, SwapperError> {
        let api = LayerZeroScanApi::new(_provider.clone());
        let response = api.get_message_by_tx(_transaction_hash).await?;
        let messages = response.data;
        let message = messages.first().ok_or(SwapperError::NetworkError {
            msg: "Unable to check transaction status using Stargate Provider: No message found".into(),
        })?;
        Ok(message.status.is_delivered())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_contain_all_endpoints() {
        let stargate = Stargate::new();
        assert_eq!(
            stargate.enpoints,
            vec![
                STARGATE_ROUTES.ethereum.clone(),
                STARGATE_ROUTES.base.clone(),
                STARGATE_ROUTES.optimism.clone(),
                STARGATE_ROUTES.arbitrum.clone(),
                STARGATE_ROUTES.polygon.clone(),
                STARGATE_ROUTES.avalanche.clone(),
                STARGATE_ROUTES.linea.clone(),
                STARGATE_ROUTES.smartchain.clone(),
                STARGATE_ROUTES.sei.clone(),
                STARGATE_ROUTES.mantle.clone(),
            ]
        );
    }
}
