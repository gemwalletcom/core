#[typeshare(swift = "Sendable")]
struct StellarAccount {
    sequence: Option<String>,
    balances: Option<Vec<StellarBalance>>,
    status: Option<i32>,
}

#[typeshare(swift = "Sendable")]
struct StellarBalance {
    balance: String,
    asset_type: String,
}

#[typeshare(swift = "Sendable")]
struct StellarFees {
    last_ledger_base_fee: String,
    fee_charged: StellarFeeCharged,
}

#[typeshare(swift = "Sendable")]
struct StellarFeeCharged {
    min: String,
    p95: String,
}

#[typeshare(swift = "Sendable")]
struct StellarNodeStatus {
    ingest_latest_ledger: i32,
    network_passphrase: String,
}

#[typeshare(swift = "Sendable")]
struct StellarTransactionBroadcast {
    hash: Option<String>,
    #[serde(rename = "title")]
    error_message: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct StellarTransactionStatus {
    successful: bool,
    fee_charged: String,
}
