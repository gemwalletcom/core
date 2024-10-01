#[typeshare(swift = "Sendable")]
struct XRPResult<T> {
    result: T,
}

#[typeshare(swift = "Sendable")]
struct XRPAccountResult {
    account_data: XRPAccount,
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
    median_fee: String,
}

#[typeshare(swift = "Sendable")]
struct XRPTransactionBroadcast {
    accepted: bool,
    engine_result_message: Option<String>,
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
