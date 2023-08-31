#[typeshare]
struct EthereumTransaction {
    status: String,
}

#[typeshare]
struct EthereumFeeHistory {
    reward: Vec<Vec<String>>,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: Vec<String>
}