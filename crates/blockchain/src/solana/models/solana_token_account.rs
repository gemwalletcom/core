#[typeshare(swift = "Sendable")]
struct SolanaTokenAccountPubkey {
    pubkey: String,
}

// accounts
#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaStakeAccount {
    account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaStakeInfo>>>,
    pubkey: String,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaTokenAccount {
    account: SolanaAccount<SolanaAccountParsed<SolanaAccountParsedInfo<SolanaTokenInfo>>>,
}

// parsed data

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaAccount<T> {
    lamports: Int,
    space: i32,
    data: T,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaAccountParsed<T> {
    parsed: T,
}

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
struct SolanaAccountParsedInfo<T> {
    info: T,
}

// parsed data: stake
#[typeshare(swift = "Sendable")]
struct SolanaStakeInfo {
    stake: SolanaStake,
    meta: SolanaRentExemptReserve,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaRentExemptReserve {
    rent_exempt_reserve: String,
}

#[typeshare(swift = "Sendable")]
struct SolanaStake {
    delegation: SolanaStakeDelegation,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaStakeDelegation {
    voter: String,
    stake: String,
    activation_epoch: String,
    deactivation_epoch: String,
}

// // parsed data: token
#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SolanaTokenInfo {
    token_amount: SolanaTokenAmount,
}

#[typeshare(swift = "Sendable")]
struct SolanaTokenAmount {
    amount: String,
}
