use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::GasPriceType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferDataOutputType {
    EncodedTransaction,
    Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDataExtra {
    pub gas_limit: Option<BigInt>,
    pub gas_price: Option<GasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: TransferDataOutputType,
}

impl Default for TransferDataExtra {
    fn default() -> Self {
        Self {
            gas_limit: None,
            gas_price: None,
            data: None,
            output_type: TransferDataOutputType::EncodedTransaction,
        }
    }
}