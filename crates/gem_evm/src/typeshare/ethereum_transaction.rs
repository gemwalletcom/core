use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
pub struct EthereumTransactionReciept {
    pub status: String,
    pub gas_used: String,
    pub effective_gas_price: String,
    #[serde(rename = "l1Fee")]
    pub l1_fee: Option<String>,
}