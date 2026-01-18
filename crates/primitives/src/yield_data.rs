use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub enum YieldAction {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
pub struct YieldData {
    pub provider_name: String,
    pub contract_address: String,
    pub call_data: String,
}
