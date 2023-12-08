#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct Delegation {
    pub balance: String,
    pub validator: DelegationValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct DelegationValidator {
    pub id: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum DelegationState {
    Active,
    Pending,
}
