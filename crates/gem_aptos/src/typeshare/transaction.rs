use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosTransaction {
    pub success: bool,
    pub gas_used: String,
    pub gas_unit_price: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosTransactionBroacast {
    pub hash: String,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosSignature {
    pub r#type: String,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosTransactionPayload {
    pub arguments: Vec<String>,
    pub function: String,
    pub r#type: String,
    pub type_arguments: Vec<String>,
}

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosTransactionSimulation {
    pub expiration_timestamp_secs: String,
    pub gas_unit_price: String,
    pub max_gas_amount: String,
    pub payload: AptosTransactionPayload,
    pub sender: String,
    pub sequence_number: String,
    pub signature: AptosSignature,
}
