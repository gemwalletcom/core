#[typeshare]
struct AssetData {
    asset: Asset,
    balance: Balance,
    price: Option<Price>,
    metadata: AssetMetaData
}