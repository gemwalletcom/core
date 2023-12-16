use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare]
pub struct UTXO {
    pub transaction_id: String,
    pub vout: i32,
    pub value: String,
}
