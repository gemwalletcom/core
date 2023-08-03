#[typeshare(swift = "Equatable, Codable")]
#[serde(rename_all = "camelCase")]
struct Account {
    chain: Chain,
    address: String,
    derivation_path: String,
    extended_public_key: Option<String>
}