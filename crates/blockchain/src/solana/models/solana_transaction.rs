#[typeshare(swift = "Sendable")]
struct SolanaTransaction {
    meta: SolanaTransactionMeta,
    slot: i32,
}

#[typeshare(swift = "Sendable")]
struct SolanaTransactionMeta {
    err: Option<SolanaTransactionError>,
}

#[typeshare(swift = "Sendable")]
struct SolanaTransactionError {}
