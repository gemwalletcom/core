#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct XRPResult<T> {
    result: T,
}

#[typeshare(swift = "Sendable")]
struct XRPAccountResult {
    account_data: Option<XRPAccount>,
}

#[typeshare(swift = "Sendable")]
struct XRPAccount {
    #[serde(rename = "Balance")]
    balance: String,
    #[serde(rename = "Sequence")]
    sequence: i32,
}

#[typeshare(swift = "Sendable")]
struct XRPFee {
    drops: XRPDrops,
}

#[typeshare(swift = "Sendable")]
struct XRPDrops {
    minimum_fee: String,
    median_fee: String,
}

#[typeshare(swift = "Sendable")]
struct XRPTransactionBroadcast {
    accepted: Option<bool>,
    engine_result_message: Option<String>,
    error_exception: Option<String>,
    tx_json: Option<XRPTransaction>,
}

#[typeshare(swift = "Sendable")]
struct XRPTransaction {
    hash: String,
}

#[typeshare(swift = "Sendable")]
struct XRPTransactionStatus {
    status: String,
}

#[typeshare(swift = "Sendable")]
struct XRPLatestBlock {
    ledger_current_index: Int,
}

#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[typeshare(swift = "Sendable")]
struct XRPAccountObjects<T> {
    account_objects: T,
}

#[serde(rename_all = "PascalCase")]
#[typeshare(swift = "Sendable")]
struct XRPAccountAsset {
    low_limit: XRPAssetLine,
}

#[typeshare(swift = "Sendable")]
struct XRPAssetLine {
    currency: String,
}

#[typeshare(swift = "Sendable")]
struct XRPTokenId {
    issuer: String,
    currency: String,
}
