use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_bigint_from_str;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateOrderResponse {
    pub estimation: DlnEstimation,
    pub order: DlnOrder,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateOrderDataResponse {
    pub order_id: String,
    pub tx: DlnTx,
    pub estimation: DlnEstimation,
    pub order: DlnOrder,
    pub fix_fee: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DlnTx {
    pub allowance_target: Option<String>,
    pub allowance_value: Option<String>,
    pub to: Option<String>,
    pub data: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DlnEstimation {
    pub src_chain_token_in: DlnTokenIn,
    pub src_chain_token_out: Option<DlnTokenIn>,
    pub dst_chain_token_out: DlnTokenOut,
    pub recommended_slippage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DlnTokenIn {
    pub address: String,
    pub chain_id: Option<u64>,
    pub decimals: i32,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DlnTokenOut {
    pub address: String,
    pub chain_id: Option<u64>,
    pub decimals: i32,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub amount: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub max_theoretical_amount: BigInt,
    #[serde(deserialize_with = "deserialize_bigint_from_str")]
    pub recommended_amount: BigInt,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DlnOrder {
    pub approximate_fulfillment_delay: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateOrderRequest {
    pub src_chain_id: String,
    pub src_chain_token_in: String,
    pub src_chain_token_in_amount: String,
    pub dst_chain_id: String,
    pub dst_chain_token_out: String,
    pub dst_chain_token_out_amount: String,
    pub dst_chain_token_out_recipient: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_chain_order_authority_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_chain_order_authority_address: Option<String>,
    pub affiliate_fee_percent: f64,
    pub affiliate_fee_recipient: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrderStatus {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChainToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChainTokenWithMinAmount {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i32,
    pub amount: String,
    pub min_amount: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_create_order_response() {
        let json_data = include_str!("test/create_order_data.json");
        let response: CreateOrderDataResponse = serde_json::from_str(json_data).expect("Failed to deserialize DlnResponse");

        assert!(!response.order_id.is_empty(), "Order ID should not be empty");
        assert!(response.tx.data.is_some(), "Transaction data should be present");

        let json_data = include_str!("test/create_order.json");
        let response: CreateOrderResponse = serde_json::from_str(json_data).expect("Failed to deserialize DlnResponse");

        assert!(response.estimation.dst_chain_token_out.amount > BigInt::from(0), "Amount should not be empty");
    }
}
