use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WCEthereumTransaction {
    pub chain_id: Option<String>,
    pub from: String,
    pub to: String,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub gas_limit: Option<String>,
    pub gas_price: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: Option<String>,
}
