#[typeshare]
struct TronAccountRequest {
    address: String,
    visible: bool,
}

#[typeshare]
struct TronAccount {
    balance: u32,
    address: Option<String>,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct TronAccountUsage {
    free_net_used: Option<i32>,
    free_net_limit: i32,
}

#[typeshare]
struct TronEmptyAccount {
    address: Option<String>,
}
