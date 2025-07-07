use super::{bs58_to_hex, encode_parameters, hex_to_utf8, model::*};
use crate::network::{AlienProvider, AlienTarget};
use crate::swapper::SwapperError;
use alloy_primitives::U256;
use primitives::Chain;
use std::sync::Arc;

#[derive(Debug)]
pub struct TronClient {
    provider: Arc<dyn AlienProvider>,
}

impl TronClient {
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self { provider }
    }

    pub async fn get_token_allowance(&self, owner_address: &str, token_address: &str, spender_address: &str) -> Result<U256, SwapperError> {
        let owner_hex = bs58_to_hex(owner_address)?;
        let spender_hex = bs58_to_hex(spender_address)?;
        let parameter = encode_parameters(&owner_hex, &spender_hex);
        let params = serde_json::json! (
            {
                "owner_address": owner_address,
                "contract_address": token_address,
                "function_selector": "allowance(address,address)",
                "parameter": hex::encode(&parameter),
                "visible": true
            }
        );

        let endpoint = self.provider.get_endpoint(Chain::Tron).map_err(SwapperError::from)?;
        let url = format!("{endpoint}/wallet/triggerconstantcontract");
        let target = AlienTarget::post_json(&url, params);
        let data = self.provider.request(target).await.map_err(SwapperError::from)?;
        let response: TronNodeResponse = serde_json::from_slice(&data).map_err(SwapperError::from)?;

        match response.result {
            TronNodeResult::Result(TronResult { result }) => {
                if !result {
                    return Err(SwapperError::NetworkError("Check approval failed. result is false".into()));
                }
                let constant_result = response
                    .constant_result
                    .first()
                    .ok_or_else(|| SwapperError::ABIError("Missing constant_result in TronGrid response".into()))?;
                let allowance = U256::from_str_radix(constant_result, 16).map_err(SwapperError::from)?;
                Ok(allowance)
            }
            TronNodeResult::Error(TronErrorResult { code, message }) => {
                let msg = format!("Check approval failed. Code: {}, Message: {}", code, hex_to_utf8(&message).unwrap_or_default());
                Err(SwapperError::NetworkError(msg))
            }
        }
    }

    pub async fn estimate_energy(
        &self,
        owner_address: &str,
        contract_address: &str,
        function_selector: &str,
        parameter: &str,
        fee_limit: u64,
        call_value: &str,
    ) -> Result<u64, SwapperError> {
        let params = serde_json::json! (
            {
                "owner_address": owner_address,
                "contract_address": contract_address,
                "function_selector": function_selector,
                "parameter": parameter,
                "fee_limit": fee_limit,
                "call_value": call_value.parse::<u64>().unwrap_or_default(),
                "visible": true
            }
        );

        let endpoint = self.provider.get_endpoint(Chain::Tron).map_err(SwapperError::from)?;
        let url = format!("{endpoint}/wallet/triggerconstantcontract");
        let target = AlienTarget::post_json(&url, params);
        let data = self.provider.request(target).await.map_err(SwapperError::from)?;

        let response: TronNodeResponse = serde_json::from_slice(&data).map_err(|e| SwapperError::NetworkError(e.to_string()))?;

        match response.result {
            TronNodeResult::Error(TronErrorResult { code, message }) => {
                let msg = format!("Estimate energy failed. Code: {}, Message: {}", code, hex_to_utf8(&message).unwrap_or_default());
                Err(SwapperError::NetworkError(msg))
            }
            TronNodeResult::Result(TronResult { result }) => {
                if !result {
                    Err(SwapperError::NetworkError("Estimate energy failed".to_string()))
                } else {
                    Ok(response.energy_used + response.energy_penalty)
                }
            }
        }
    }
}
