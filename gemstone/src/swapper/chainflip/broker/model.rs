use serde::{Deserialize, Serialize};

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
