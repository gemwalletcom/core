#[typeshare(swift = "Sendable")]
struct CosmosAccount {
    account_number: String,
    sequence: String,
}

#[typeshare(swift = "Sendable")]
struct CosmosAccountResponse<T> {
    account: T,
}

#[typeshare(swift = "Sendable")]
struct CosmosInjectiveAccount {
    base_account: CosmosAccount,
}
