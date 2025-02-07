use std::{collections::HashMap, str::FromStr, sync::Arc};

use alloy_core::sol_types::{SolCall, SolValue};
use alloy_primitives::{hex, Address, Bytes, FixedBytes, U160, U256};
use gem_evm::{
    erc20::IERC20,
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    multicall_handler::{Call, Instructions},
    stargate::contract::{IStargate, MessagingFee, OFTReceipt, SendParam},
};
use primitives::{AssetId, Chain};

use crate::{
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{slippage::apply_slippage_in_bp, SwapQuoteRequest, SwapperError},
};

use super::endpoint::{StargateEndpoint, StargatePool};

fn build_extra_options(compose_msg: &[u8]) -> Bytes {
    if compose_msg.is_empty() {
        return Bytes::new();
    }

    let mut options = vec![0x00, 0x03]; // Type 3 header

    // Add LZ Compose Option (index 0, 200k gas, 0 value)
    let worker_id = 0x01u8;
    let option_type = 0x03u8;

    // Compose option data: index (u16) + gas (u128) + value (u128)
    let index = 0u16.to_be_bytes();
    let gas = 200_000u128.to_be_bytes();
    let _value = 0u128.to_be_bytes();

    let mut option_data = Vec::with_capacity(34);
    option_data.extend_from_slice(&index);
    option_data.extend_from_slice(&gas);
    //option_data.extend_from_slice(&value);

    // Worker ID (1 byte) + option length (2 bytes) + option type (1 byte)
    options.push(worker_id);
    let option_length = (option_data.len() + 1) as u16; // +1 for option_type
    options.extend_from_slice(&option_length.to_be_bytes());
    options.push(option_type);
    options.extend_from_slice(&option_data);

    Bytes::from(options)
}

#[derive(Debug)]
pub struct StargateOFTQuote {
    pub receipt: OFTReceipt,
}

#[derive(Debug, Default)]
pub struct StargateClient {
    endpoints: Vec<StargateEndpoint>,
    chain_endpoints: HashMap<Chain, StargateEndpoint>,
    pools: HashMap<AssetId, StargatePool>,
}

impl StargateClient {
    pub fn from_endpoints(endpoints: Vec<StargateEndpoint>) -> Self {
        let mut chain_endpoints = HashMap::new();
        let mut pools = HashMap::new();

        for endpoint in endpoints.iter() {
            chain_endpoints.insert(endpoint.id, endpoint.clone());
            for pool in endpoint.pools.iter() {
                pools.insert(pool.asset.id.clone(), pool.clone());
            }
        }

        Self {
            endpoints,
            chain_endpoints,
            pools,
        }
    }

    pub fn address_to_bytes32(&self, addr: &str) -> FixedBytes<32> {
        FixedBytes::<32>::from(U256::from(U160::from_str(addr).unwrap()))
    }

    pub fn get_endpoints(&self) -> Vec<&StargateEndpoint> {
        self.endpoints.iter().collect()
    }

    pub fn get_endpoint_by_chain(&self, chain: &Chain) -> Result<&StargateEndpoint, SwapperError> {
        self.chain_endpoints.get(chain).ok_or(SwapperError::NotSupportedChain)
    }

    pub fn get_pool_by_asset_id(&self, asset: &AssetId) -> Result<&StargatePool, SwapperError> {
        self.pools.get(asset).ok_or(SwapperError::NotSupportedChain)
    }

    pub fn get_decimals_by_asset_id(&self, asset: &AssetId) -> Result<i32, SwapperError> {
        self.get_pool_by_asset_id(asset).map(|p| p.asset.decimals)
    }

    pub fn build_compose_msg(&self, amount: U256, request: &SwapQuoteRequest) -> Result<Vec<u8>, SwapperError> {
        let fees = request.options.fee.as_ref().map(|f| f.evm_bridge.clone());
        let fee_bps = fees.clone().map(|f| f.bps).unwrap_or(0);
        let amount_after_fee = apply_slippage_in_bp(&amount, fee_bps);

        let destination_address = Address::from_str(request.destination_address.as_str())?;
        let referrer_address = fees.clone().map(|f| Address::from_str(f.address.clone().as_str()).unwrap()).unwrap();

        let fee_amount = amount - amount_after_fee;
        let oft_token = if request.to_asset.is_native() {
            Address::ZERO
        } else {
            Address::from_str(request.to_asset.token_id.as_ref().unwrap().as_ref()).unwrap()
        };

        let instruction_calls = if request.from_asset.is_native() {
            vec![
                Call {
                    target: destination_address,
                    callData: Bytes::new(),
                    value: amount_after_fee,
                },
                Call {
                    target: referrer_address,
                    callData: Bytes::new(),
                    value: fee_amount,
                },
            ]
        } else {
            vec![
                Call {
                    target: oft_token,
                    callData: IERC20::transferCall {
                        to: destination_address,
                        value: amount_after_fee,
                    }
                    .abi_encode()
                    .into(),
                    value: U256::ZERO,
                },
                Call {
                    target: oft_token,
                    callData: IERC20::transferCall {
                        to: referrer_address,
                        value: fee_amount,
                    }
                    .abi_encode()
                    .into(),
                    value: U256::ZERO,
                },
            ]
        };

        let instructions = Instructions {
            calls: instruction_calls.clone(),
            fallbackRecipient: destination_address,
        };

        let compose_msg = instructions.abi_encode();

        Ok(compose_msg)
    }

    pub fn build_send_param(&self, request: &SwapQuoteRequest) -> Result<SendParam, SwapperError> {
        let amount_ld = U256::from_str(request.value.as_str()).unwrap();
        let destination_endpoint = self.get_endpoint_by_chain(&request.to_asset.chain)?;
        let fees = request.options.fee.as_ref().map(|f| f.evm_bridge.clone());
        let fee_bps = fees.clone().map(|f| f.bps).unwrap_or(0);
        let amount_after_fee = apply_slippage_in_bp(&amount_ld, fee_bps);

        let compose_msg = self.build_compose_msg(amount_ld, request)?;
        let extra_options = build_extra_options(&compose_msg);

        Ok(SendParam {
            dstEid: destination_endpoint.endpoint_id,
            // TODO: Move composer address to pool struct
            to: self.address_to_bytes32(destination_endpoint.composer_address.as_str()),
            amountLD: amount_ld,
            minAmountLD: amount_after_fee,
            extraOptions: extra_options,
            composeMsg: compose_msg.into(),
            oftCmd: Bytes::new(),
        })
    }

    pub async fn quote_oft(&self, pool: &StargatePool, send_param: &SendParam, provider: Arc<dyn AlienProvider>) -> Result<StargateOFTQuote, SwapperError> {
        let calldata = IStargate::quoteOFTCall {
            _sendParam: send_param.clone(),
        }
        .abi_encode();

        let call = EthereumRpc::Call(TransactionObject::new_call(pool.address.as_str(), calldata), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, provider, &pool.asset.chain()).await?;
        let decoded = hex::decode(response.take()?).map_err(|e| SwapperError::NetworkError {
            msg: format!("Unable to hex decode quote oft response: {:?}", e.to_string()),
        })?;
        let returns = IStargate::quoteOFTCall::abi_decode_returns(&decoded, true).map_err(|e| SwapperError::ABIError {
            msg: format!("Unable to abi decode quote oft response: {:?}", e.to_string()),
        })?;

        Ok(StargateOFTQuote { receipt: returns.receipt })
    }

    pub async fn quote_send(&self, pool: &StargatePool, send_param: &SendParam, provider: Arc<dyn AlienProvider>) -> Result<MessagingFee, SwapperError> {
        let calldata = IStargate::quoteSendCall {
            _sendParam: send_param.clone(),
            _payInLzToken: false,
        }
        .abi_encode();

        let call = EthereumRpc::Call(TransactionObject::new_call(pool.address.as_str(), calldata), BlockParameter::Latest);
        let response: JsonRpcResult<String> = jsonrpc_call(&call, provider, &pool.asset.chain()).await?;
        let decoded = hex::decode(response.take()?).map_err(|e| SwapperError::NetworkError {
            msg: format!("Unable to hex decode quote send response: {:?}", e.to_string()),
        })?;
        let returns = IStargate::quoteSendCall::abi_decode_returns(&decoded, true).map_err(|e| SwapperError::ABIError {
            msg: format!("Unable to abi decode quote send response: {:?}", e.to_string()),
        })?;

        println!("quote_send_response: {:?}", returns);

        Ok(returns.fee)
    }

    pub fn send(&self, send_param: &SendParam, fee: &MessagingFee, refund_address: &Address) -> Vec<u8> {
        IStargate::sendCall {
            _sendParam: send_param.clone(),
            _fee: fee.clone(),
            _refundAddress: *refund_address,
        }
        .abi_encode()
    }
}

#[cfg(test)]
mod tests {
    use crate::swapper::stargate::endpoint::STARGATE_ROUTES;

    use super::*;

    #[test]
    fn test_get_endpoint_id() {
        let stargate = StargateClient::from_endpoints(vec![STARGATE_ROUTES.ethereum.clone()]);

        // Test valid chain
        let result = stargate.get_endpoint_by_chain(&Chain::Ethereum);
        assert!(result.is_ok());

        // Test invalid chain
        let result = stargate.get_endpoint_by_chain(&Chain::Manta);
        assert!(matches!(result, Err(SwapperError::NotSupportedChain)));
    }

    #[test]
    fn test_address_to_bytes32() {
        let stargate = StargateClient::from_endpoints(vec![STARGATE_ROUTES.ethereum.clone()]);
        let test_address = "0x0655c6AbdA5e2a5241aa08486bd50Cf7d475CF24";
        let test_result = "0x0000000000000000000000000655c6abda5e2a5241aa08486bd50cf7d475cf24";
        let result = stargate.address_to_bytes32(test_address);

        assert_eq!(result.len(), 32);
        assert_eq!(result, FixedBytes::<32>::from_str(test_result).unwrap());
    }

    #[test]
    fn test_get_pool() {
        let stargate = StargateClient::from_endpoints(vec![STARGATE_ROUTES.ethereum.clone()]);

        // Test with valid asset
        let valid_asset = AssetId::from_chain(Chain::Ethereum); // Add appropriate asset details
        let result = stargate.get_pool_by_asset_id(&valid_asset);
        assert!(result.is_ok());

        // Test with invalid asset
        let invalid_asset = AssetId::from_chain(Chain::Manta);
        let result = stargate.get_pool_by_asset_id(&invalid_asset);
        assert!(matches!(result, Err(SwapperError::NotSupportedChain)));
    }

    #[test]
    fn test_get_asset_decimals() {
        let stargate = StargateClient::from_endpoints(vec![STARGATE_ROUTES.ethereum.clone()]);

        // Test with valid asset
        let valid_asset = AssetId::from_chain(Chain::Ethereum); // Add appropriate asset details
        let result = stargate.get_decimals_by_asset_id(&valid_asset);
        assert!(result.is_ok());

        // Test with invalid asset
        let invalid_asset = AssetId::from_chain(Chain::Manta);
        let result = stargate.get_decimals_by_asset_id(&invalid_asset);
        assert!(matches!(result, Err(SwapperError::NotSupportedChain)));
    }
}
