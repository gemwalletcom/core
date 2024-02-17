#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct Delegation {
    pub base: DelegationBase,
    pub validator: DelegationValidator,
    pub price: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationBase {
    pub asset_id: AssetId,
    pub state: DelegationState,
    pub balance: String,
    pub rewards: String,
    pub completion_date: Option<Date>,
    pub delegation_id: String,
    pub validator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct DelegationValidator {
    pub chain: Chain,
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub commision: f64,
    pub apr: f64,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum DelegationState {
    Active,
    Pending,
    Undelegating,
    Inactive,
    Activating,
    Deactivating,
    AwaitingWithdrawal,
}
