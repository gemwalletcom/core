#[typeshare(swift = "Sendable")]
struct TonWalletInfo {
    seqno: Option<i32>,
    last_transaction_id: TonTransactionId,
}
#[typeshare(swift = "Sendable")]
struct TonResult<T> {
    result: T,
}
