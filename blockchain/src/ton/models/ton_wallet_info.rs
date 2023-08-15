#[typeshare]
struct TonWalletInfo {
    seqno: Option<i32>,
    last_transaction_id: TonTransactionId,
}