use std::{collections::HashMap, str::FromStr, sync::Arc};

use alloy_core::sol_types::SolCall;
use alloy_primitives::{hex, Address, Bytes, FixedBytes, U160, U256};
use gem_evm::{
    jsonrpc::{BlockParameter, EthereumRpc, TransactionObject},
    stargate::contract::{IStargate, MessagingFee, OFTReceipt, SendParam},
};
use primitives::{AssetId, Chain};

use crate::{
    network::{jsonrpc_call, AlienProvider, JsonRpcResult},
    swapper::{SwapQuoteRequest, SwapperError},
};

use super::endpoint::{StargateEndpoint, StargatePool};

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

    pub fn build_send_param(&self, request: &SwapQuoteRequest) -> Result<SendParam, SwapperError> {
        let from_asset = &request.from_asset;
        let to_asset = &request.to_asset;

        if from_asset.chain == to_asset.chain {
            return Err(SwapperError::NotSupportedPair);
        }

        if from_asset.is_native() && !to_asset.is_native() {
            return Err(SwapperError::NotSupportedPair);
        }

        let amount_ld = U256::from_str(request.value.as_str()).unwrap();

        let destination_endpoint = self.get_endpoint_by_chain(&to_asset.chain)?;

        Ok(SendParam {
            dstEid: destination_endpoint.endpoint_id,
            to: self.address_to_bytes32(request.destination_address.as_str()),
            amountLD: amount_ld,
            minAmountLD: amount_ld,
            extraOptions: Bytes::new(),
            composeMsg: Bytes::new(),
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
