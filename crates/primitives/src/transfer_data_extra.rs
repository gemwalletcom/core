use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

use crate::{GasPriceType, TransferDataOutputAction, TransferDataOutputType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDataExtra {
    pub to: String,
    pub gas_limit: Option<BigInt>,
    pub gas_price: Option<GasPriceType>,
    pub data: Option<Vec<u8>>,
    pub output_type: TransferDataOutputType,
    pub output_action: TransferDataOutputAction,
}

impl Default for TransferDataExtra {
    fn default() -> Self {
        Self {
            to: "".to_string(),
            gas_limit: None,
            gas_price: None,
            data: None,
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Send,
        }
    }
}
