#[typeshare]
struct AssetData {
    asset: Asset,
    balance: Balance,
    account: Account,
    price: Option<Price>,
    details: Option<AssetDetailsInfo>,
    metadata: AssetMetaData,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
struct AssetAddress {
    asset: Asset,
    address: String,
}
