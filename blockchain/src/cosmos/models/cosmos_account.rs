#[typeshare]
struct CosmosAccount {
    account_number: String,
    sequence: String,
}

#[typeshare]
struct CosmosAccountResponse {
    account: CosmosAccount,
}
