use crate::models::custom_types::GemBigInt;
use primitives::{
    AssetId, SimulationBalanceChange, SimulationHeader, SimulationPayloadField, SimulationPayloadFieldDisplay, SimulationPayloadFieldKind, SimulationPayloadFieldType,
    SimulationResult, SimulationSeverity, SimulationWarning, SimulationWarningType,
};

#[uniffi::remote(Enum)]
pub enum SimulationSeverity {
    Low,
    Warning,
    Critical,
}

#[uniffi::remote(Enum)]
pub enum SimulationWarningType {
    TokenApproval { asset_id: AssetId, value: Option<GemBigInt> },
    SuspiciousSpender,
    ExternallyOwnedSpender,
    NftCollectionApproval { asset_id: AssetId },
    PermitApproval { asset_id: AssetId, value: Option<GemBigInt> },
    PermitBatchApproval { value: Option<GemBigInt> },
    ValidationError,
}

#[uniffi::remote(Record)]
pub struct SimulationWarning {
    pub severity: SimulationSeverity,
    pub warning: SimulationWarningType,
    pub message: Option<String>,
}

#[uniffi::remote(Record)]
pub struct SimulationBalanceChange {
    pub asset_id: AssetId,
    pub value: String,
}

#[uniffi::remote(Enum)]
pub enum SimulationPayloadFieldType {
    Text,
    Address,
    Timestamp,
}

#[uniffi::remote(Enum)]
pub enum SimulationPayloadFieldDisplay {
    Primary,
    Secondary,
}

#[uniffi::remote(Enum)]
pub enum SimulationPayloadFieldKind {
    Contract,
    Method,
    Token,
    Spender,
    Value,
    Custom,
}

#[uniffi::remote(Record)]
pub struct SimulationPayloadField {
    pub kind: SimulationPayloadFieldKind,
    pub label: Option<String>,
    pub value: String,
    pub field_type: SimulationPayloadFieldType,
    pub display: SimulationPayloadFieldDisplay,
}

#[uniffi::remote(Record)]
pub struct SimulationHeader {
    pub asset_id: AssetId,
    pub value: String,
    pub is_unlimited: bool,
}

#[uniffi::remote(Record)]
pub struct SimulationResult {
    pub warnings: Vec<SimulationWarning>,
    pub balance_changes: Vec<SimulationBalanceChange>,
    pub payload: Vec<SimulationPayloadField>,
    pub header: Option<SimulationHeader>,
}
