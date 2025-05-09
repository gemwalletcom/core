use alloy_primitives::U256;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_hex_str, serialize_biguint_to_hex_str};

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
    pub min_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaParameters {
    pub number_of_chunks: u32,
    pub chunk_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapEvmExtras {
    pub chain: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub input_amount: BigUint,
    pub refund_parameters: RefundParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultSwapResponse {
    pub calldata: String,
    #[serde(deserialize_with = "deserialize_biguint_from_hex_str", serialize_with = "serialize_biguint_to_hex_str")]
    pub value: BigUint,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainflipEnvironment {
    pub ingress_egress: ChainflipIngressEgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainflipIngressEgress {
    pub minimum_deposit_amounts: serde_json::Value,
}

impl ChainflipIngressEgress {
    pub fn get_min_deposit_amount(&self, asset: &ChainflipAsset) -> Result<U256, SwapperError> {
        let chain_map = self.minimum_deposit_amounts.get(&asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let asset = chain_map.get(&asset.asset).ok_or(SwapperError::NotSupportedAsset)?;
        let amount = asset.as_str().ok_or(SwapperError::NotSupportedAsset)?;

        let u256_value = U256::from_str_radix(amount.trim_start_matches("0x"), 16).map_err(SwapperError::from)?;
        Ok(u256_value)
    }
}
