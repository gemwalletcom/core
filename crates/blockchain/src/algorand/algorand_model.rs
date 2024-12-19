#[typeshare(swift = "Sendable")]
struct AlgorandAccount {
    amount: UInt64,
}

#[typeshare(swift = "Sendable")]
struct AlgorandTransactionParams {
    #[serde(rename = "min-fee")]
    min_fee: i32,
    #[serde(rename = "genesis-id")]
    genesis_id: String,
    #[serde(rename = "genesis-hash")]
    genesis_hash: String,
    #[serde(rename = "last-round")]
    last_round: i32,
}

#[typeshare(swift = "Sendable")]
struct AlgorandVersions {
    #[serde(rename = "genesis-id")]
    genesis_id: String,
    #[serde(rename = "genesis-hash")]
    genesis_hash: String,
}

#[typeshare(swift = "Sendable")]
struct AlgorandTransactionBroadcast {
    #[serde(rename = "txId")]
    tx_id: String,
}

#[typeshare(swift = "Sendable")]
struct AlgorandTransactionBroadcastError {
    message: String,
}

#[typeshare(swift = "Sendable")]
struct AlgorandTransactionStatus {
    #[serde(rename = "confirmed-round")]
    confirmed_round: i32,
}
