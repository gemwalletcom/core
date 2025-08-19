use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaBlockhashResult {
    pub value: SolanaBlockhash,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaBlockhash {
    pub blockhash: String,
}
