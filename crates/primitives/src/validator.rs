#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct StakeValidator {
    pub id: String,
    pub name: String,
}
