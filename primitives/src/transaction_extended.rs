#[typeshare]
struct TransactionExtended {
    transaction: Transaction,
    asset: Asset,
    #[serde(rename = "feeAsset")]
    feeAsset: Asset,
    price: Option<Price>,
    #[serde(rename = "feePrice")]
    fee_price: Option<Price>,
}