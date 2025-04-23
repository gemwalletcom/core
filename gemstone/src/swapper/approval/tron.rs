use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use bs58;
use gem_evm::erc20::IERC20;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::network::{AlienProvider, AlienTarget};
use crate::swapper::{models::ApprovalType, ApprovalData, SwapperError};
use primitives::Chain;

#[derive(Debug, Deserialize)]
struct TronGridResponse {
    result: TronGridResult,
    constant_result: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TronGridResult {
    Result(TronResult),
    Error(TronErrorResult),
}

#[derive(Deserialize, Debug)]
struct TronResult {
    result: bool,
}

#[derive(Deserialize, Debug)]
struct TronErrorResult {
    code: String,
    message: String,
}

fn bs58_to_hex(address: &str) -> Result<Vec<u8>, SwapperError> {
    bs58::decode(address)
        .with_check(None)
        .into_vec()
        .map_err(|e| SwapperError::InvalidAddress(format!("Failed to decode address '{}': {}", address, e)))
}

fn hex_to_utf8(hex: &str) -> Option<String> {
    hex::decode(hex).ok().and_then(|bytes| String::from_utf8(bytes).ok())
}

fn encode_parameters(owner: &[u8], spender: &[u8]) -> Vec<u8> {
    let owner_addr = Address::from_slice(&owner[1..]);
    let spender_addr = Address::from_slice(&spender[1..]);
    let parameter = IERC20::allowanceCall {
        owner: owner_addr,
        spender: spender_addr,
    }
    .abi_encode();
    parameter[4..].to_vec() // drop function selector
}

pub async fn check_approval_tron(
    owner_address: &str,
    token_address: &str,
    spender_address: &str,
    amount: U256,
    provider: Arc<dyn AlienProvider>,
    chain: &Chain,
) -> Result<ApprovalType, SwapperError> {
    // 1. Encode the function call parameters

    let owner_hex = bs58_to_hex(owner_address)?;
    let spender_hex = bs58_to_hex(spender_address)?;
    let parameter = encode_parameters(&owner_hex, &spender_hex);

    // 2. Construct the request payload
    let params = json! (
        {
            "owner_address": owner_address,
            "contract_address": token_address,
            "function_selector": "allowance(address,address)",
            "parameter": hex::encode(&parameter),
            "visible": true
        }
    );

    let endpoint = provider.get_endpoint(*chain).map_err(SwapperError::from)?;
    let url = format!("{}/wallet/triggerconstantcontract", endpoint);
    let target = AlienTarget::post_json(&url, params);
    let data = provider.request(target).await.map_err(SwapperError::from)?;

    // 4. Parse response
    let response: TronGridResponse = serde_json::from_slice(&data).map_err(SwapperError::from)?;

    if let TronGridResult::Error(TronErrorResult { code, message }) = response.result {
        let msg = format!("Check approval failed. Code: {}, Message: {}", code, hex_to_utf8(&message).unwrap_or_default());
        return Err(SwapperError::NetworkError(msg));
    };

    if let TronGridResult::Result(TronResult { result }) = response.result {
        if !result {
            return Err(SwapperError::NetworkError("Check approval failed. result is false".into()));
        }
        // 5. Extract and decode the allowance amount
        let constant_result = response
            .constant_result
            .first()
            .ok_or_else(|| SwapperError::ABIError("Missing constant_result in TronGrid response".into()))?;

        let allowance = U256::from_str_radix(constant_result, 16).map_err(SwapperError::from)?;

        // 6. Compare allowance and return result
        return if allowance < amount {
            Ok(ApprovalType::Approve(ApprovalData {
                token: token_address.to_string(),
                spender: spender_address.to_string(),
                value: amount.to_string(),
            }))
        } else {
            Ok(ApprovalType::None)
        };
    }

    Err(SwapperError::NetworkError("Failed to parse TronGrid response".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tron_encoding() {
        let token_address = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";
        let owner_address = "TA7mCjHFfo68FG3wc6pDCeRGbJSPZkBfL7";
        let gateway_address = "TQjjYNyBmzCyDh5WumFJBhXFyE5PUKqVYZ";

        let token_hex = bs58_to_hex(token_address).unwrap();
        let owner_hex = bs58_to_hex(owner_address).unwrap();
        let gateway_hex = bs58_to_hex(gateway_address).unwrap();

        assert_eq!(hex::encode(&token_hex), "41a614f803b6fd780986a42c78ec9c7f77e6ded13c");
        assert_eq!(hex::encode(&owner_hex), "41019e353a35efaa8e27c2a602a791ae1b19d9c9fa");
        assert_eq!(hex::encode(&gateway_hex), "41a1fd8e8afc126545d76b4a9e905d5be1ccd392e1");

        let parameter = encode_parameters(&owner_hex, &gateway_hex);

        assert_eq!(
            hex::encode(&parameter),
            "000000000000000000000000019e353a35efaa8e27c2a602a791ae1b19d9c9fa000000000000000000000000a1fd8e8afc126545d76b4a9e905d5be1ccd392e1"
        );
    }
}
