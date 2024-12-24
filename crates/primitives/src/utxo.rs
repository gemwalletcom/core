use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable")]
pub struct UTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
    pub address: Option<String>,
}
