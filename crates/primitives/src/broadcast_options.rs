use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[typeshare(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct BroadcastOptions {
    pub skip_preflight: bool,
    pub from_address: Option<String>,
}

impl BroadcastOptions {
    pub fn new(skip_preflight: bool) -> Self {
        Self {
            skip_preflight,
            from_address: None,
        }
    }
}
