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
}

#[typeshare]
struct SolanaStakeAccountDataParsedInfoStake {
    delegation: SolanaStakeAccountDataParsedInfoStakeDelegation,
}

#[typeshare]
struct SolanaStakeAccountDataParsedInfoStakeDelegation {
    voter: String,
    stake: String,
}
