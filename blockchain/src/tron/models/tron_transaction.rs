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