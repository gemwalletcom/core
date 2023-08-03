#[typeshare]
struct TonTransaction {
    transaction_id: TonTransactionId,
}

#[typeshare]
struct TonTransactionId {
    hash: String,
}