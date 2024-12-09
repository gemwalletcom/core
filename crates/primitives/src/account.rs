#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
struct Account {
    chain: Chain,
    address: String,
    derivation_path: String,
    public_key: Option<String>,
    extended_public_key: Option<String>,
}
