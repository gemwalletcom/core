#[typeshare]
struct TonWalletInfo {
    seqno: Option<i32>,
    last_transaction_id: TonTransactionId,
}
#[typeshare]
struct TonResult<T> {
    result: T,
}
