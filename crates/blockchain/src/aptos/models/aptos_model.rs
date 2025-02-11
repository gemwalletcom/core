use typeshare::typeshare;
//use serde::{Serialize, Deserialize};

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[allow(dead_code)]
struct AptosResource<T> {
    r#type: String,
    data: T,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosResourceBalance {
    coin: AptosResourceCoin,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosResourceBalanceOptional {
    coin: Option<AptosResourceCoin>,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosResourceCoin {
    value: String,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosAccount {
    sequence_number: String,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosTransaction {
    success: bool,
    gas_used: String,
    gas_unit_price: String,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosTransactionBroacast {
    hash: String,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosGasFee {
    deprioritized_gas_estimate: i32,
    gas_estimate: i32,
    prioritized_gas_estimate: i32,
}

#[typeshare(swift = "Sendable")]
#[allow(dead_code)]
struct AptosLedger {
    chain_id: i32,
    ledger_version: String,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosCoinInfo {
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosSignature {
    pub r#type: String,
    pub public_key: Option<String>,
    pub signature: Option<String>,
}

#[typeshare(swift = "Equatable, Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosTransactionPayload {
    pub arguments: Vec<String>,
    pub function: String,
    pub r#type: String,
    pub type_arguments: Vec<String>,
}

#[typeshare(swift = "Equatable, Sendable")]
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
