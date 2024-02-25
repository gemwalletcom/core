#[typeshare]
struct SolanaValidators {
    current: Vec<SolanaValidator>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaValidator {
    vote_pubkey: String,
    commission: i32,
    epoch_vote_account: bool,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaEpoch {
    epoch: i32,
}
