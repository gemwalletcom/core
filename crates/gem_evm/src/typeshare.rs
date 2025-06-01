use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct EthereumTransactionReciept {
    status: String,
    gas_used: String,
    effective_gas_price: String,
    #[serde(rename = "l1Fee")]
    l1_fee: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct EthereumFeeHistory {
    reward: Vec<Vec<String>>,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Vec<String>,
}
