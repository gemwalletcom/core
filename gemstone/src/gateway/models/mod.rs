pub mod asset;
pub mod balances;
pub mod perpetual;
pub mod staking;
pub mod transaction;

pub use balances::*;
pub use perpetual::*;
pub use staking::*;
pub use transaction::*;

// Re-export simpler models inline
use primitives::{FeePriorityValue, UTXO};

// ChainAccount models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemUTXO {
    pub transaction_id: String,
    pub vout: u32,
    pub value: String,
    pub address: String,
}

// ChainState models
#[derive(Debug, Clone, uniffi::Record)]
pub struct GemFeePriorityValue {
    pub priority: String,
    pub value: String,
}

// Conversion implementations
impl From<UTXO> for GemUTXO {
    fn from(utxo: UTXO) -> Self {
        Self {
            transaction_id: utxo.transaction_id,
            vout: utxo.vout as u32,
            value: utxo.value,
            address: utxo.address,
        }
    }
}

impl From<FeePriorityValue> for GemFeePriorityValue {
    fn from(fee: FeePriorityValue) -> Self {
        Self {
            priority: fee.priority.as_ref().to_string(),
            value: fee.value,
        }
    }
}
