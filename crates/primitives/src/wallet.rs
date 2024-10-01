#[typeshare(swift = "Equatable, Sendable")]
struct Wallet {
    id: String,
    name: String,
    index: i32,
    #[serde(rename = "type")]
    wallet_type: WalletType,
    accounts: Vec<Account>,
    order: i32,
    #[serde(rename = "isPinned")]
    is_pinned: bool,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum WalletType {
    multicoin,
    single,
    private_key,
    view,
}

#[typeshare(swift = "Equatable, Sendable")]
struct WalletId {
    id: String,
}
