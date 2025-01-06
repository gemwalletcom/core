#[typeshare(swift = "Sendable")]
struct StellarAccount {
    sequence: String,
    balances: Vec<StellarBalance>,
}

#[typeshare(swift = "Sendable")]
struct StellarBalance {
    balance: String,
    asset_type: String,
    asset_code: Option<String>,
    asset_issuer: Option<String>,
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

#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
struct StellarEmbedded<T> {
    _embedded: StellarRecords<T>,
}

#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
struct StellarRecords<T> {
    records: Vec<T>,
}

#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
struct StellarAsset {
    asset_code: String,
    asset_issuer: String,
    contract_id: Option<String>,
}
