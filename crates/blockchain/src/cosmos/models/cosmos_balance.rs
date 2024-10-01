#[typeshare(swift = "Sendable")]
struct CosmosBalances {
    balances: Vec<CosmosBalance>,
}

#[typeshare(swift = "Sendable")]
struct CosmosBalance {
    denom: String,
    amount: String,
}
