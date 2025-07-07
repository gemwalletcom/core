use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[typeshare(swiftGenericConstraints = "T: Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuiData<T> {
    pub data: T,
}
