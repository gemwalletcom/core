#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiCoinBalance {
    coin_type: String,
    total_balance: UInt64, 
}