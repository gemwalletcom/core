#[typeshare(swift = "Sendable")]
struct SolanaBalance {
    value: Int,
}

#[typeshare(swift = "Sendable")]
struct SolanaBalanceValue {
    amount: String,
}
