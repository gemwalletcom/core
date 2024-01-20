#[typeshare]
struct SolanaTokenAccount {
    pubkey: String,
}

#[typeshare]
struct SolanaTokenAccountResult<T> {
    account: T,
    pubkey: String,
}

#[typeshare]
struct SolanaStakeAccount {
    lamports: i32,
    space: i32,
    data: SolanaStakeAccountData,
}

#[typeshare]
struct SolanaStakeAccountData {
    parsed: SolanaStakeAccountDataParsed,
}

#[typeshare]
struct SolanaStakeAccountDataParsed {
    info: SolanaStakeAccountDataParsedInfo,
}

#[typeshare]
struct SolanaStakeAccountDataParsedInfo {
    stake: SolanaStakeAccountDataParsedInfoStake,
    meta: SolanaStakeAccountDataParsedInfoMeta,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaStakeAccountDataParsedInfoMeta {
    rent_exempt_reserve: String,
}

#[typeshare]
struct SolanaStakeAccountDataParsedInfoStake {
    delegation: SolanaStakeAccountDataParsedInfoStakeDelegation,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SolanaStakeAccountDataParsedInfoStakeDelegation {
    voter: String,
    stake: String,
    activation_epoch: String,
    deactivation_epoch: String,
}
