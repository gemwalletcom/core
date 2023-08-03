#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaBlockhashResult {
    value: SolanaBlockhash,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaBlockhash {
    blockhash: String,
}