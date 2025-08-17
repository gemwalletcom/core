use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearGasPrice {
    pub gas_price: String,
}
