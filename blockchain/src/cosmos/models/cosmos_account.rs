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
struct CosmosIbjectiveAccount {
    base_account: CosmosAccount,
}
