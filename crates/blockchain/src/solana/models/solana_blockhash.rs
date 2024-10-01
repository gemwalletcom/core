#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaBlockhashResult {
    value: SolanaBlockhash,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaBlockhash {
    blockhash: String,
}
