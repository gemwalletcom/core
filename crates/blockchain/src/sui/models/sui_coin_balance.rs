#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiCoin {
    coin_type: String,
    coin_object_id: String,
    balance: String,
    version: String,
    digest: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiCoinBalance {
    coin_type: String,
    total_balance: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiTransaction {
    effects: SuiEffects,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiStatus {
    status: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiEffects {
    gas_used: SuiGasUsed,
    status: SuiStatus,
    created: Option<Vec<SuiObjectChange>>,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiObjectChange {
    reference: SuiObjectReference,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiObjectReference {
    object_id: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiGasUsed {
    computationCost: String,
    storageCost: String,
    storageRebate: String,
    nonRefundableStorageFee: String,
}

#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct SuiData<T> {
    data: T,
}
