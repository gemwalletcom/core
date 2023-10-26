#[typeshare(swift = "Equatable, Codable, Hashable")]
#[serde(rename_all = "camelCase")]
struct WCEthereumTransaction {
    from: String,
    to: String,
    value: Option<String>,
    gas: Option<String>,
    gas_limit: Option<String>,
    gas_price: Option<String>,
    nonce: Option<String>,
    data: Option<String>,
}