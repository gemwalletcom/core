#[typeshare(swift = "Sendable")]
struct TronTransactionBroadcast {
    result: bool,
    txid: String,
}

#[typeshare(swift = "Sendable")]
struct TronTransactionBroadcastError {
    message: String,
}

#[typeshare(swift = "Sendable")]
struct TronTransaction {
    ret: Vec<TronTransactionContractRef>,
}

#[typeshare(swift = "Sendable")]
struct TronTransactionContractRef {
    contractRet: String,
}

#[typeshare(swift = "Sendable")]
struct TronTransactionReceipt {
    #[serde(rename = "blockNumber")]
    pub block_number: i32,
    fee: Option<i32>,
    result: Option<String>,
    receipt: Option<TronReceipt>,
}

#[typeshare(swift = "Sendable")]
struct TronReceipt {
    result: Option<String>,
}
