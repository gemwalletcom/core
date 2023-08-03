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
struct TronEmptyAccount {
    address: Option<String>,
}