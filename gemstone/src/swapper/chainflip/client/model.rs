use num_bigint::BigUint;
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub amount: String,
    pub src_chain: String,
    pub src_asset: String,
    pub dest_chain: String,
    pub dest_asset: String,
    pub is_vault_swap: bool,
    pub dca_enabled: bool,
    pub broker_commission_bps: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncludedFee {
    #[serde(rename = "type")]
    pub fee_type: String,
    pub chain: String,
    pub asset: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DcaParams {
    pub number_of_chunks: u32,
    pub chunk_interval_blocks: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    #[serde(rename = "type")]
    pub quote_type: String,
    pub deposit_amount: String,
    pub is_vault_swap: bool,
    pub boost_quote: Option<BoostQuote>,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostQuote {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub egress_amount: BigUint,
    pub recommended_slippage_tolerance_percent: f64,
    pub estimated_duration_seconds: f64,
    pub estimated_boost_fee_bps: u32,
    pub max_boost_fee_bps: u32,
    pub estimated_price: String,
    pub dca_params: Option<DcaParams>,
}

impl QuoteResponse {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}

impl BoostQuote {
    pub fn slippage_bps(&self) -> u32 {
        (self.recommended_slippage_tolerance_percent * 100.0) as u32
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapTxResponse {
    pub state: String,
    pub swap_id: String,
    pub dest_chain: String,
    pub swap_egress: Option<SwapEgress>,
}

impl SwapTxResponse {
    pub fn swap_status(&self) -> SwapStatus {
        match self.state.as_str() {
            "COMPLETED" => SwapStatus::Completed,
            "FAILED" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapEgress {
    pub tx_ref: Option<String>,
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn get_quote_response() {
        let json = include_str!("./test/btc_eth_quote.json");
        let quote_response = serde_json::from_str::<Vec<QuoteResponse>>(json).unwrap();

        assert!(quote_response[0].boost_quote.is_some());
    }
}
