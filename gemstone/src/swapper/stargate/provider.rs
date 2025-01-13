use std::str::FromStr;
use std::sync::Arc;

use alloy_core::sol_types::SolCall;
use alloy_primitives::{hex, Address, Bytes, FixedBytes, U160, U256, U32};
use async_trait::async_trait;
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    stargate::contract::{IRouter, MessagingFee, SendParam},
};
use primitives::{AssetId, Chain};
use serde::{Deserialize, Serialize};

use crate::{
    debug_println,
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{
        approval::check_approval_erc20,
        asset::{BASE_USDC, ETHEREUM_USDC, ETHEREUM_USDT, OPTIMISM_USDC},
        eth_rpc,
        slippage::apply_slippage_in_bp,
        ApprovalType, FetchQuoteData, GemSwapProvider, SwapChainAsset, SwapProvider, SwapProviderData, SwapQuote, SwapQuoteData, SwapQuoteRequest, SwapRoute,
        SwapperError,
    },
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct StargateRouteData {
    send_param: SendParam,
    fee: MessagingFee,
    refund_address: String,
}

#[derive(Debug, Default)]
pub struct Stargate {}

impl Stargate {
    pub fn get_endpoint_id(&self, chain: &Chain) -> u32 {
        match chain {
            Chain::Ethereum => 30101u32,
            Chain::Optimism => 30111u32,
            Chain::Base => 30184u32,
            _ => 0u32,
        }
    }

    pub fn address_to_bytes32(&self, addr: &str) -> FixedBytes<32> {
        FixedBytes::<32>::from(U256::from(U160::from_str(addr).unwrap()))
    }

    pub fn get_pool(&self, asset: &AssetId) -> Option<String> {
        match asset.chain {
            Chain::Base => match &asset.token_id {
                Some(token_id) => Some("0x27a16dc786820b16e5c9028b75b99f6f604b5d26".to_string()),
                None => Some("0xdc181Bd607330aeeBEF6ea62e03e5e1Fb4B6F7C7".to_string()),
            },
            Chain::Optimism => match &asset.token_id {
                Some(token_id) => Some("0xcE8CcA271Ebc0533920C83d39F417ED6A0abB7D0".to_string()),
                None => Some("0xe8CDF27AcD73a434D661C84887215F7598e7d0d3".to_string()),
            },
            _ => None,
        }
    }
}

#[async_trait]
impl GemSwapProvider for Stargate {
    fn provider(&self) -> SwapProvider {
        SwapProvider::Stargate
    }

    fn supported_assets(&self) -> Vec<SwapChainAsset> {
        Chain::all()
            .iter()
            .map(|chain| match chain {
                Chain::Base => SwapChainAsset::Assets(chain.clone(), vec![BASE_USDC.id.clone()]),
                Chain::Optimism => SwapChainAsset::Assets(chain.clone(), vec![OPTIMISM_USDC.id.clone()]),
                _ => SwapChainAsset::Assets(chain.clone(), vec![]),
            })
            .collect()
    }

    async fn fetch_quote(&self, request: &SwapQuoteRequest, provider: Arc<dyn AlienProvider>) -> Result<SwapQuote, SwapperError> {
        println!("request: {:?}", request);
        let pool = self.get_pool(&request.from_asset).unwrap();
        let amount_ld = U256::from_str(request.value.as_str()).unwrap();
        let mut send_param = SendParam {
            dstEid: self.get_endpoint_id(&request.to_asset.chain),
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
        let call_data = IRouter::quoteOFTCall {
            _sendParam: send_param.clone(),
        }
        .abi_encode();

        let call = EthereumRpc::Call(TransactionObject::new_call(pool.as_str(), call_data), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, provider.clone(), &request.from_asset.chain).await?;
        let result = response.take()?;
        let hex_data = hex::decode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        let quote_oft_data = IRouter::quoteOFTCall::abi_decode_returns(&hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;

        println!("quote oft - {:?}", quote_oft_data);
        //println!("feeAmount = {}", quote_oft_data.oftFeeDetails[0].feeAmountLD);
        send_param.minAmountLD = apply_slippage_in_bp(&quote_oft_data.receipt.amountReceivedLD, request.options.slippage_bps);
        //send_param.minAmountLD = U256::from(99500u32);

        let messaging_fee_calldata = IRouter::quoteSendCall {
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
            IRouter::quoteSendCall::abi_decode_returns(&messaging_fee_hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
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
        let send_calldata = IRouter::sendCall {
            _sendParam: route_data.send_param.clone(),
            _fee: route_data.fee.clone(),
            _refundAddress: Address::from_str(route_data.refund_address.as_str()).unwrap(),
        }
        .abi_encode();
        //
        //let send_calldata = hex::decode("0xc7c7f5b3000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000e021ca97a2f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000655c6abda5e2a5241aa08486bd50cf7d475cf24000000000000000000000000000000000000000000000000000000000000759f0000000000000000000000000655c6abda5e2a5241aa08486bd50cf7d475cf2400000000000000000000000000000000000000000000000000000000000186a000000000000000000000000000000000000000000000000000000000000184ac00000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();

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
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        network::{provider::AlienProvider, target::*, *},
        swapper::SwapperError,
    };
    use alloy_core::{
        hex::{decode as HexDecode, encode_prefixed as HexEncode},
        primitives::{Address, Bytes, FixedBytes, U160, U256},
        sol_types::{SolCall, SolValue},
    };
    use alloy_primitives::utils::parse_units;
    use async_trait::async_trait;
    use futures::TryFutureExt;
    use gem_evm::{
        jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
        stargate::contract::{
            IRouter::{self, IRouterCalls},
            LzTxObj, SendParam,
        },
    };
    use primitives::{Asset, AssetId, Chain};
    use reqwest::Client;
    use std::{collections::HashMap, sync::Arc};

    use super::*;

    #[derive(Debug)]
    pub struct NativeProvider {
        pub node_config: HashMap<Chain, String>,
        pub client: Client,
    }

    impl NativeProvider {
        pub fn new(node_config: HashMap<Chain, String>) -> Self {
            Self {
                node_config,
                client: Client::new(),
            }
        }
    }

    #[async_trait]
    impl AlienProvider for NativeProvider {
        fn get_endpoint(&self, chain: Chain) -> Result<String, AlienError> {
            Ok(self
                .node_config
                .get(&chain)
                .ok_or(AlienError::ResponseError {
                    msg: "not supported chain".into(),
                })?
                .to_string())
        }

        async fn request(&self, target: AlienTarget) -> Result<Data, AlienError> {
            println!("==> request: url: {:?}, method: {:?}", target.url, target.method);
            let mut req = match target.method {
                AlienHttpMethod::Get => self.client.get(target.url),
                AlienHttpMethod::Post => self.client.post(target.url),
                AlienHttpMethod::Put => self.client.put(target.url),
                AlienHttpMethod::Delete => self.client.delete(target.url),
                AlienHttpMethod::Head => self.client.head(target.url),
                AlienHttpMethod::Patch => self.client.patch(target.url),
                AlienHttpMethod::Options => todo!(),
            };
            if let Some(headers) = target.headers {
                for (key, value) in headers.iter() {
                    req = req.header(key, value);
                }
            }
            if let Some(body) = target.body {
                println!("==> request body size: {:?}", body.len());
                println!("==> request body: {:?}", String::from_utf8(body.clone()).unwrap());
                req = req.body(body);
            }

            let response = req
                .send()
                .map_err(|e| AlienError::ResponseError {
                    msg: format!("reqwest send error: {:?}", e),
                })
                .await?;
            let bytes = response
                .bytes()
                .map_err(|e| AlienError::ResponseError {
                    msg: format!("request error: {:?}", e),
                })
                .await?;

            println!("<== response body size: {:?}", bytes.len());
            Ok(bytes.to_vec())
        }

        async fn batch_request(&self, targets: Vec<AlienTarget>) -> Result<Vec<Data>, AlienError> {
            let mut futures = vec![];
            for target in targets.iter() {
                let future = self.request(target.clone());
                futures.push(future);
            }
            let responses = futures::future::join_all(futures).await;
            let error = responses.iter().find_map(|x| x.as_ref().err());
            if let Some(err) = error {
                return Err(err.clone());
            }
            let responses = responses.into_iter().filter_map(|x| x.ok()).collect();
            Ok(responses)
        }
    }

    fn address_to_bytes32(addr: &str) -> FixedBytes<32> {
        FixedBytes::<32>::from(U256::from(U160::from_str(addr).unwrap()))
    }

    #[tokio::test]
    async fn test_swap_usdc_base_to_usdc_op() -> Result<(), SwapperError> {
        let node_config = HashMap::from([(Chain::Base, "https://mainnet.base.org".to_string())]);
        let network_provider = Arc::new(NativeProvider::new(node_config));

        let op_dst_eid = 30111u32;
        let amount_ld = U256::from(1_000_000_000u64);

        let mut send_param = SendParam {
            dstEid: op_dst_eid,
            to: address_to_bytes32("0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24"),
            amountLD: amount_ld,
            minAmountLD: amount_ld,
            extraOptions: Bytes::from_str("0x").unwrap(),
            composeMsg: Bytes::from_str("0x").unwrap(),
            oftCmd: Bytes::from_str("0x").unwrap(),
        };
        println!("send_param: {:?}", send_param);

        // Encode call data
        let call_data = IRouter::quoteOFTCall {
            _sendParam: send_param.clone(),
        }
        .abi_encode();

        let call = EthereumRpc::Call(
            TransactionObject::new_call("0x27a16dc786820b16e5c9028b75b99f6f604b5d26", call_data),
            BlockParameter::Latest,
        );
        let response: JsonRpcResult<String> = jsonrpc_call(&call, network_provider.clone(), &Chain::Base).await?;
        let result = response.take()?;
        let hex_data = HexDecode(result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        println!("quoteLayerZeroFee eth_call result: {:?}", hex_data);

        let value = IRouter::quoteOFTCall::abi_decode_returns(&hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;

        println!("nativeFee = {}", value.receipt.amountSentLD);
        println!("zroFee    = {}", value.receipt.amountReceivedLD);
        println!("feeAmount = {}", value.oftFeeDetails[0].feeAmountLD);
        send_param.minAmountLD = value.receipt.amountSentLD;

        let messaging_fee_calldata = IRouter::quoteSendCall {
            _sendParam: send_param.clone(),
            _payInLzToken: false,
        }
        .abi_encode();

        let messaging_fee_call = EthereumRpc::Call(
            TransactionObject::new_call("0x27a16dc786820b16e5c9028b75b99f6f604b5d26", messaging_fee_calldata),
            BlockParameter::Latest,
        );
        let messaging_fee_response: JsonRpcResult<String> = jsonrpc_call(&messaging_fee_call, network_provider.clone(), &Chain::Base).await?;
        let messaging_fee_result = messaging_fee_response.take()?;
        let messaging_fee_hex_data = HexDecode(messaging_fee_result).map_err(|e| SwapperError::NetworkError { msg: e.to_string() })?;
        println!("messagingFee eth_call result: {:?}", messaging_fee_hex_data);

        let messaging_fee_value =
            IRouter::quoteSendCall::abi_decode_returns(&messaging_fee_hex_data, true).map_err(|e| SwapperError::ABIError { msg: e.to_string() })?;
        println!("messagingFee amountSentLD = {}", messaging_fee_value.fee.nativeFee);
        println!("messagingFee amountReceivedLD = {}", messaging_fee_value.fee.lzTokenFee);
        //
        // --------------------------------------------------
        // 2) swap(...) via signed raw transaction
        // --------------------------------------------------
        // Hypothetical pool IDs for USDC(Base)->USDC(OP)
        //let src_pool_id = U256::from(1);
        //let dst_pool_id = U256::from(2);
        //let amount_ld = U256::from(50_000_000u64); // 50 USDC
        //let min_amount_ld = U256::from(49_000_000u64);
        //
        //// Refund address
        //let refund_address = Address::from_slice(&hex_decode("0000000000000000000000000123456789abCDef0123456789AbCdef01234567").unwrap());

        //let swap_data = IRouter::swapCall {
        //    _dstChainId: dst_chain_id,
        //    _srcPoolId: src_pool_id,
        //    _dstPoolId: dst_pool_id,
        //    _refundAddress:
        //    _amountLD: amount_ld,
        //    _minAmountLD: min_amount_ld,
        //    _lzTxParams: lz_obj.clone(),
        //    _to: to_addr_bytes.clone(),
        //    _payload: payload.clone(),
        //}
        //.abi_encode();
        //
        //// We need nonce & gasPrice
        //// Derive "from" address from the private key if you want to do an actual state-changing tx
        //// For brevity, let's just assume we know the address:
        //let from_addr_hex = "0x0123456789abCDef0123456789abcDEF012345678";
        //let nonce = get_transaction_count(&client, rpc_url, from_addr_hex).await?;
        //let gas_price_biguint = get_gas_price(&client, rpc_url).await?;
        //println!("nonce = {}, gasPrice = {}", nonce, gas_price_biguint);
        //
        //let nonce_u256 = U256::from(nonce);
        //let gas_price_u256 = U256::from(gas_price_biguint);
        //let gas_limit = U256::from(2_000_000u64);
        //
        //// The bridging fee is paid in Base ETH, so transaction "value" = `native_fee`
        //let tx_value = native_fee;
        //
        //// Sign a LEGACY TX (many chains use EIP-1559, but this is just a demonstration)
        //let raw_tx = sign_legacy_tx(
        //    chain_id,
        //    nonce_u256,
        //    gas_price_u256,
        //    gas_limit,
        //    router_addr_20,
        //    tx_value,
        //    swap_data,
        //    &signing_key,
        //);
        //
        //// Send raw TX
        //let tx_hash = send_raw_transaction(&client, rpc_url, raw_tx).await?;
        //println!("swap() transaction submitted! txHash = {}", tx_hash);

        Ok(())
    }
}
