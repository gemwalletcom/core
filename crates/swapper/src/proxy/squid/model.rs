use primitives::swap::SwapStatus;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidTransactionStatus {
    pub squid_transaction_status: SquidStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SquidStatus {
    Success,
    Ongoing,
    PartialSuccess,
    NeedsGas,
    NotFound,
}

impl SquidStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            SquidStatus::Success | SquidStatus::PartialSuccess => SwapStatus::Completed,
            SquidStatus::Ongoing | SquidStatus::NeedsGas | SquidStatus::NotFound => SwapStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_status_response() {
        let result: SquidTransactionStatus = serde_json::from_str(include_str!("../../../testdata/squid/status_response.json")).unwrap();
        assert_eq!(result.squid_transaction_status, SquidStatus::Success);
        assert_eq!(result.squid_transaction_status.swap_status(), SwapStatus::Completed);
    }
}
