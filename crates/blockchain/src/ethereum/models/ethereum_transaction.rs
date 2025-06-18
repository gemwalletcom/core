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
#[serde(rename_all = "camelCase")]
struct EthereumFeeHistory {
    pub reward: Vec<Vec<String>>,
    pub base_fee_per_gas: Vec<String>,
    pub gas_used_ratio: Vec<f64>,
    pub oldest_block: String,
}
