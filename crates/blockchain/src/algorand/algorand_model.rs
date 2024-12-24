#[typeshare(swift = "Sendable")]
struct AlgorandAccount {
    amount: UInt64,
    assets: Vec<AlgorandAccountAccount>,
    #[serde(rename = "min-balance")]
    min_balance: i32,
}

#[typeshare(swift = "Sendable")]
struct AlgorandAccountAccount {
    amount: UInt64,
    #[serde(rename = "asset-id")]
    asset_id: i32,
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
    tx_id: Option<String>,
    #[serde(rename = "message")]
    error_message: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct AlgorandTransactionStatus {
    #[serde(rename = "confirmed-round")]
    confirmed_round: i32,
}

#[typeshare(swift = "Sendable")]
struct AlgorandAssetResponse {
    params: AlgorandAsset,
}

#[typeshare(swift = "Sendable")]
struct AlgorandAsset {
    decimals: i32,
    name: String,
    #[serde(rename = "unit-name")]
    unit_name: String,
}
