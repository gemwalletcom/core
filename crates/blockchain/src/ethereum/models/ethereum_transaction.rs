#[typeshare]
#[serde(rename_all = "camelCase")]
struct EthereumTransactionReciept {
    status: String,
    gas_used: String,
    effective_gas_price: String,
    #[serde(rename = "l1Fee")]
    l1_fee: Option<String>,
}

#[typeshare]
struct EthereumFeeHistory {
    reward: Vec<Vec<String>>,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Vec<String>,
}
