#[typeshare]
struct XRPAccountResponse {
    result: XRPAccountResult
}

#[typeshare]
struct XRPAccountResult {
    account_data: XRPAccount
}

#[typeshare]
struct XRPAccount {
    #[serde(rename = "Balance")]
    balance: String,
}