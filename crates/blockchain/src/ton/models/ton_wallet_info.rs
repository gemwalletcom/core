#[typeshare(swift = "Sendable")]
struct TonWalletInfo {
    seqno: Option<i32>,
    last_transaction_id: TonTransactionId,
}
#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct TonResult<T> {
    result: T,
}
