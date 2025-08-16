use primitives::{TransactionChange, TransactionMetadata, TransactionPerpetualMetadata, TransactionStateRequest, TransactionUpdate};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionUpdate {
    pub state: String,
    pub changes: Vec<GemTransactionChange>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionChange {
    HashChange { old: String, new: String },
    Metadata(GemTransactionMetadata),
    BlockNumber(String),
    NetworkFee(String),
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemTransactionMetadata {
    Perpetual(GemTransactionPerpetualMetadata),
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionPerpetualMetadata {
    pub pnl: f64,
    pub price: f64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransactionStateRequest {
    pub id: String,
    pub sender_address: String,
    pub created_at: i64,
}

impl From<TransactionChange> for GemTransactionChange {
    fn from(value: TransactionChange) -> Self {
        match value {
            TransactionChange::HashChange { old, new } => GemTransactionChange::HashChange { old, new },
            TransactionChange::Metadata(metadata) => GemTransactionChange::Metadata(metadata.into()),
            TransactionChange::BlockNumber(block_number) => GemTransactionChange::BlockNumber(block_number),
            TransactionChange::NetworkFee(fee) => GemTransactionChange::NetworkFee(fee),
        }
    }
}

impl From<TransactionMetadata> for GemTransactionMetadata {
    fn from(value: TransactionMetadata) -> Self {
        match value {
            TransactionMetadata::Perpetual(perp) => GemTransactionMetadata::Perpetual(perp.into()),
        }
    }
}

impl From<TransactionPerpetualMetadata> for GemTransactionPerpetualMetadata {
    fn from(value: TransactionPerpetualMetadata) -> Self {
        GemTransactionPerpetualMetadata {
            pnl: value.pnl,
            price: value.price,
        }
    }
}

impl From<GemTransactionStateRequest> for TransactionStateRequest {
    fn from(value: GemTransactionStateRequest) -> Self {
        TransactionStateRequest {
            id: value.id,
            sender_address: value.sender_address,
            created_at: value.created_at,
        }
    }
}

impl From<TransactionUpdate> for GemTransactionUpdate {
    fn from(value: TransactionUpdate) -> Self {
        GemTransactionUpdate {
            state: value.state.to_string(),
            changes: value.changes.into_iter().map(|change| change.into()).collect(),
        }
    }
}
