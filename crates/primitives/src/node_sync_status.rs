use crate::UInt64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeSyncStatus {
    pub in_sync: bool,
    pub latest_block_number: Option<UInt64>,
    pub current_block_number: Option<UInt64>,
}

impl NodeSyncStatus {
    pub fn new(in_sync: bool, latest_block_number: Option<UInt64>, current_block_number: Option<UInt64>) -> Self {
        Self {
            in_sync,
            latest_block_number,
            current_block_number,
        }
    }

    pub fn in_sync() -> Self {
        Self::new(true, None, None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum NodeStatusState {
    Healthy(NodeSyncStatus),
    Error { message: String },
}

impl NodeStatusState {
    pub fn healthy(status: NodeSyncStatus) -> Self {
        Self::Healthy(status)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error { message: message.into() }
    }

    pub fn is_healthy(&self) -> bool {
        match self {
            Self::Healthy(status) => status.in_sync,
            Self::Error { .. } => false,
        }
    }

    pub fn as_status(&self) -> Option<&NodeSyncStatus> {
        match self {
            Self::Healthy(status) => Some(status),
            Self::Error { .. } => None,
        }
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error { message } => Some(message.as_str()),
            _ => None,
        }
    }
}
