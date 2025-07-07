use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[typeshare(swift = "Sendable")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AptosAccount {
    pub sequence_number: String,
}
