#[typeshare]
struct XRPResult<T> {
    result: T
}

#[typeshare]
struct XRPAccountResult {
    account_data: XRPAccount
}

#[typeshare]
struct XRPAccount {
    #[serde(rename = "Balance")]
    balance: String,
    #[serde(rename = "Sequence")]
    sequence: i32,
}

#[typeshare]
struct XRPFee {
    drops: XRPDrops,
}

#[typeshare]
struct XRPDrops {
    median_fee: String,
}

#[typeshare]
struct XRPTransactionBroadcast {
    accepted: bool,
    engine_result_message: Option<String>,
    tx_json: Option<XRPTransaction>,
}

#[typeshare]
struct XRPTransaction {
    hash: String,
}