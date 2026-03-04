use super::model::{RelayCurrency, RelayCurrencyDetail, RelayRequest, RelayRequestData, RelayRequestMetadata, RelayStatus};

impl RelayRequest {
    pub fn mock(status: RelayStatus, metadata: Option<RelayRequestMetadata>) -> Self {
        Self {
            status,
            data: metadata.map(|m| RelayRequestData { metadata: Some(m) }),
        }
    }
}

impl RelayCurrencyDetail {
    pub fn mock(address: &str, chain_id: u64, amount: &str) -> Self {
        Self {
            currency: RelayCurrency {
                chain_id,
                address: address.to_string(),
            },
            amount: Some(amount.to_string()),
        }
    }
}
