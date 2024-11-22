#[typeshare(swift = "Sendable")]
struct AssetData {
    asset: Asset,
    balance: Balance,
    account: Account,
    price: Option<Price>,
    price_alert: Option<PriceAlert>,
    metadata: AssetMetaData,
}

#[typeshare(swift = "Equatable, Hashable, Sendable")]
struct AssetAddress {
    asset: Asset,
    address: String,
}
