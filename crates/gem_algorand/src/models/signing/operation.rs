use crate::address::AlgorandAddress;

const TX_TYPE_PAYMENT: &str = "pay";
const TX_TYPE_ASSET_TRANSFER: &str = "axfer";

// Canonical msgpack field counts per operation type.
const PAYMENT_FIELDS: u8 = 9;
const PAYMENT_ZERO_AMOUNT_FIELDS: u8 = 8;
const ASSET_TRANSFER_FIELDS: u8 = 10;
const ASSET_OPT_IN_FIELDS: u8 = 9;

pub enum Operation {
    Payment { destination: AlgorandAddress, amount: u64 },
    AssetTransfer { destination: AlgorandAddress, amount: u64, asset_id: u64 },
    AssetOptIn { asset_id: u64 },
}

impl Operation {
    pub fn tx_type(&self) -> &'static str {
        match self {
            Self::Payment { .. } => TX_TYPE_PAYMENT,
            Self::AssetTransfer { .. } | Self::AssetOptIn { .. } => TX_TYPE_ASSET_TRANSFER,
        }
    }

    pub fn size(&self) -> u8 {
        match self {
            Self::Payment { amount: 0, .. } => PAYMENT_ZERO_AMOUNT_FIELDS,
            Self::Payment { .. } => PAYMENT_FIELDS,
            Self::AssetTransfer { .. } => ASSET_TRANSFER_FIELDS,
            Self::AssetOptIn { .. } => ASSET_OPT_IN_FIELDS,
        }
    }

    pub fn payment_amount(&self) -> Option<u64> {
        match self {
            Self::Payment { amount, .. } if *amount > 0 => Some(*amount),
            _ => None,
        }
    }
}
