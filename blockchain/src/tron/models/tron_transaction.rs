#[typeshare]
struct TronTransactionBroadcast {
    result: bool,
    txid: String,
}

#[typeshare]
struct TronTransaction {
    ret: Vec<TronTransactionContractRef>,
}

#[typeshare]
struct TronTransactionContractRef {
    contractRet: String,
}

#[typeshare]
struct TronTransactionReceipt {
    #[serde(rename = "blockNumber")]
    pub block_number: i32, 
    fee: Option<i32>,
    result: Option<String>,
    receipt: Option<TronReceipt>,
}

#[typeshare]
struct TronReceipt {
    result: Option<String>,
}