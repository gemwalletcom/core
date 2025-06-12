#[typeshare(swift = "Sendable")]
#[serde(rename_all = "camelCase")]
struct EthereumTransactionReciept {
    status: String,
    gas_used: String,
    effective_gas_price: String,
    #[serde(rename = "l1Fee")]
    l1_fee: Option<String>,
}

#[typeshare(swift = "Sendable")]
struct EthereumFeeHistory {
    reward: Vec<Vec<String>>,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Vec<String>,
    #[serde(rename = "gasUsedRatio")]
    gas_used_ratio: Vec<f64>,
}
