#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiCoin {
    coin_type: String,
    coin_object_id: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiCoinBalance {
    coin_type: String,
    total_balance: String, 
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiTransaction {
    effects: SuiEffects,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiStatus {
    status: String
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiEffects {
    gas_used: SuiGasUsed,
    status: SuiStatus,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiGasUsed {
    computationCost: String,
    storageCost: String,
    storageRebate: String,
    nonRefundableStorageFee: String,
}

#[typeshare]
#[serde(rename_all = "camelCase")]
struct SuiData<T> {
    data: T,
}
