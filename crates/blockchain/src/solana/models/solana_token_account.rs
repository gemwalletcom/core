#[typeshare(swift = "Sendable")]
struct SolanaTokenAccount {
    pubkey: String,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaTokenAccountResult<T> {
    account: T,
    pubkey: String,
}

#[typeshare(swift = "Sendable")]
struct SolanaStakeAccount {
    lamports: Int,
    space: i32,
    data: SolanaStakeAccountData,
}

#[typeshare(swift = "Sendable")]
struct SolanaStakeAccountData {
    parsed: SolanaStakeAccountDataParsed,
}

#[typeshare(swift = "Sendable")]
struct SolanaStakeAccountDataParsed {
    info: SolanaStakeAccountDataParsedInfo,
}

#[typeshare(swift = "Sendable")]
struct SolanaStakeAccountDataParsedInfo {
    stake: SolanaStakeAccountDataParsedInfoStake,
    meta: SolanaStakeAccountDataParsedInfoMeta,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaStakeAccountDataParsedInfoMeta {
    rent_exempt_reserve: String,
}

#[typeshare(swift = "Sendable")]
struct SolanaStakeAccountDataParsedInfoStake {
    delegation: SolanaStakeAccountDataParsedInfoStakeDelegation,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaStakeAccountDataParsedInfoStakeDelegation {
    voter: String,
    stake: String,
    activation_epoch: String,
    deactivation_epoch: String,
}
