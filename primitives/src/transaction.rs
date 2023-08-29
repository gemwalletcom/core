#[typeshare]
struct Transaction {
    id: String,
    hash: String,
    #[serde(rename = "assetId")]
    asset_id: AssetId,
    from: String,
    to: String,
    contract: Option<String>,
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    state: TransactionState,
    #[serde(rename = "blockNumber")]
    block_number: i32,
    sequence: i32,
    fee: String,
    #[serde(rename = "feeAssetId")]
    fee_asset_id: AssetId,
    value: String,
    memo: Option<String>,
    direction: TransactionDirection,
    #[serde(rename = "createdAt")]
    created_at: Date,
    #[serde(rename = "updatedAt")]
    updated_at: Date,
}