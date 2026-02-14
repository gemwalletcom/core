use std::fmt;

#[derive(Debug, Clone)]
pub enum NodeSwitchReason {
    BlockHeight { old_block: u64, new_block: u64 },
    Latency { old_latency_ms: u64, new_latency_ms: u64 },
    CurrentNodeError { message: String },
    AdaptiveError { error_ratio: f64, samples: usize },
}

impl NodeSwitchReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BlockHeight { .. } => "block_height",
            Self::Latency { .. } => "latency",
            Self::CurrentNodeError { .. } => "current_node_error",
            Self::AdaptiveError { .. } => "adaptive_error",
        }
    }

    pub fn metric_detail(&self) -> &'static str {
        match self {
            Self::BlockHeight { .. } => "higher_block",
            Self::Latency { .. } => "lower_latency",
            Self::CurrentNodeError { .. } => "error",
            Self::AdaptiveError { .. } => "error_ratio_threshold",
        }
    }
}

impl fmt::Display for NodeSwitchReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockHeight { old_block, new_block } => write!(f, "block_behind:{}", new_block.saturating_sub(*old_block)),
            Self::Latency { old_latency_ms, new_latency_ms } => write!(f, "latency:{}ms->{}ms", old_latency_ms, new_latency_ms),
            Self::CurrentNodeError { message } => write!(f, "{}", message),
            Self::AdaptiveError { error_ratio, samples } => write!(f, "error_ratio={:.3},samples={}", error_ratio, samples),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adaptive_reason_formats_detail() {
        let reason = NodeSwitchReason::AdaptiveError { error_ratio: 0.555, samples: 20 };
        assert_eq!(reason.as_str(), "adaptive_error");
        assert_eq!(reason.to_string(), "error_ratio=0.555,samples=20");
    }
}
