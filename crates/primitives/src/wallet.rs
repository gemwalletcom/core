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
#[serde(rename_all = "camelCase")]
pub enum WalletType {
    multicoin,
    single,
    private_key,
    view,
}

#[typeshare(swift = "Equatable, Codable")]
struct WalletId {
    id: String,
}
