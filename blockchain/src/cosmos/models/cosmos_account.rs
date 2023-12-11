#[typeshare]
struct CosmosAccount {
    account_number: String,
    sequence: String,
}

#[typeshare]
struct CosmosAccountResponse<T> {
    account: T,
}

#[typeshare]
struct CosmosInjectiveAccount {
    base_account: CosmosAccount,
}
