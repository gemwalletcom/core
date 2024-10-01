#[typeshare(swift = "Sendable")]
struct SolanaValidators {
    current: Vec<SolanaValidator>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaValidator {
    vote_pubkey: String,
    commission: i32,
    epoch_vote_account: bool,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaEpoch {
    epoch: i32,
    slot_index: i32,
    slots_in_epoch: i32,
}
