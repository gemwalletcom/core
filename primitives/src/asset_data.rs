#[typeshare]
struct AssetData {
    asset: Asset,
    balance: Balance,
    price: Option<Price>,
    details: Option<AssetDetailsInfo>,
    metadata: AssetMetaData
}