use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct StakeValidator {
    pub id: String,
    pub name: String,
}
