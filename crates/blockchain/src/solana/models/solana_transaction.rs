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

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaInstruction {
    program_id: String,
    accounts: [SolanaAccountMeta],
    data: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaAccountMeta {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}
