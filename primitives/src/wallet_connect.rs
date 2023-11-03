#[derive(Debug, Serialize)]
#[typeshare]
#[serde(rename_all = "lowercase")]
pub enum WallletConnectCAIP2 {
    #[serde(rename = "eip155")]
    Eip155,
}

#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCEthereumTransaction {
    chain_id: Option<String>,
    from: String,
    to: String,
    value: Option<String>,
    gas: Option<String>,
    gas_limit: Option<String>,
    gas_price: Option<String>,
    max_fee_per_gas: Option<String>,
    max_priority_fee_per_gas: Option<String>,
    nonce: Option<String>,
    data: Option<String>,
}
