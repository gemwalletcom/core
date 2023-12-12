#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub base: DelegationBase,
    pub validator: DelegationValidator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
#[serde(rename_all = "camelCase")]
pub struct DelegationBase {
    pub asset_id: AssetId,
    pub state: DelegationState,
    pub balance: String,
    pub completion_date: Option<Date>,
    pub delegation_id: String,
    pub validator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct DelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum DelegationState {
    Active,
    Pending,
    Undelegating,
    Rewards,
}
