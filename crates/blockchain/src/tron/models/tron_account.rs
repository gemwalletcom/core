#[typeshare(swift = "Sendable")]
struct TronAccountRequest {
    address: String,
    visible: bool,
}

#[typeshare(swift = "Sendable")]
struct TronAccount {
    balance: Option<UInt64>,
    address: Option<String>,
    active_permission: Option<Vec<TronAccountPermission>>,
}
#[typeshare(swift = "Sendable")]
struct TronAccountPermission {
    id: i32,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct TronAccountUsage {
    free_net_used: Option<i32>,
    free_net_limit: Option<i32>,
}

#[typeshare(swift = "Sendable")]
struct TronEmptyAccount {
    address: Option<String>,
}
