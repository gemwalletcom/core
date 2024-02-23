#[typeshare]
struct SolanaTransaction {
    meta: SolanaTransactionMeta,
    slot: i32,
}

#[typeshare]
struct SolanaTransactionMeta {
    err: Option<SolanaTransactionError>,
}

#[typeshare]
struct SolanaTransactionError {}
