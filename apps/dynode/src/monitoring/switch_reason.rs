use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeSwitchReason {
    BlockHeight { old_block: u64, new_block: u64 },
    Latency { old_latency_ms: u64, new_latency_ms: u64 },
    CurrentNodeError { message: String },
}

impl NodeSwitchReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BlockHeight { .. } => "block_height",
            Self::Latency { .. } => "latency",
            Self::CurrentNodeError { .. } => "current_node_error",
        }
    }
}

impl fmt::Display for NodeSwitchReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockHeight { old_block, new_block } => write!(f, "block_behind:{}", new_block.saturating_sub(*old_block)),
            Self::Latency { old_latency_ms, new_latency_ms } => write!(f, "latency:{}ms->{}ms", old_latency_ms, new_latency_ms),
            Self::CurrentNodeError { message } => write!(f, "{}", message),
        }
    }
}
