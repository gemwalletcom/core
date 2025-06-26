#[typeshare(swift = "Equatable, Sendable, Hashable")]
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
    #[serde(rename = "imageUrl")]
    image_url: Option<String>,
    #[serde(rename = "creationType")]
    creation_type: WalletCreationType,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum WalletType {
    multicoin,
    single,
    private_key,
    view,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum WalletCreationType {
    imported,
    created,
}

#[typeshare(swift = "Equatable, Sendable")]
struct WalletId {
    id: String,
}
