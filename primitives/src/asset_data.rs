#[typeshare]
struct AssetData {
    asset: Asset,
    balance: Balance,
    account: Account,
    price: Option<Price>,
    details: Option<AssetDetailsInfo>,
    metadata: AssetMetaData,
}
