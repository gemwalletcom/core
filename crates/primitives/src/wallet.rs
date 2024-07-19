#[typeshare(swift = "Equatable, Codable")]
struct Wallet {
    id: String,
    name: String,
    index: i32,
    #[serde(rename = "type")]
    wallet_type: WalletType,
    accounts: Vec<Account>,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
pub enum WalletType {
    multicoin,
    single,
    view,
}

#[typeshare(swift = "Equatable, Codable")]
struct WalletId {
    id: String,
}
