use alloy_primitives::U256;
use serde::{Deserialize, Serialize};

use crate::swapper::SwapperError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ChainflipAsset {
    pub chain: String,
    pub asset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAddressResponse {
    pub address: String,
    pub expiry_block: u64,
    pub issued_block: u64,
    pub channel_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RefundParameters {
    pub retry_duration: u32,
    pub refund_address: String,
    pub min_price: String, // U256 in hex string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaParameters {
    pub number_of_chunks: u32,
    pub chunk_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapExtraParams {
    pub chain: String,
    pub input_amount: u128,
    pub refund_parameters: RefundParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapResponse {
    pub calldata: String,
    pub value: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainflipEnvironment {
    pub ingress_egress: CahinflipIngressEgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CahinflipIngressEgress {
    pub minimum_deposit_amounts: serde_json::Value,
}

impl CahinflipIngressEgress {
    pub fn get_min_deposit_amount(&self, asset: &ChainflipAsset) -> Result<U256, SwapperError> {
        let chain_map = self.minimum_deposit_amounts.get(&asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let asset = chain_map.get(&asset.asset).ok_or(SwapperError::NotSupportedAsset)?;
        let amount = asset.as_str().ok_or(SwapperError::NotSupportedAsset)?;

        let u256_value = U256::from_str_radix(amount, 16).map_err(SwapperError::from)?;
        Ok(u256_value)
    }
}
