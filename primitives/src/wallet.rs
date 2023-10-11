#[typeshare(swift = "Equatable, Codable")]
struct Wallet {
    id: String,
    name: String,
    index: i32,
    #[serde(rename = "type")]
    wallet_type: WalletType,
    accounts: Vec<Account>
}

#[typeshare(swift = "Equatable, Codable")]
pub enum WalletType {
    multicoin,
    single,
    view,
}