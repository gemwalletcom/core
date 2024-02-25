#[typeshare]
struct CosmosBalances {
    balances: Vec<CosmosBalance>
}

#[typeshare]
struct CosmosBalance {
    denom: String,
    amount: String,
}