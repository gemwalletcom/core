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
