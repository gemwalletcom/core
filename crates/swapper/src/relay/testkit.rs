use super::model::{EvmStepData, RelayCurrency, RelayCurrencyDetail, RelayRequest, RelayRequestData, RelayRequestMetadata, RelayStatus, Step, StepData, StepItem};

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

impl Step {
    pub fn mock_transaction(id: &str, to: &str, value: &str, data: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: "transaction".to_string(),
            items: Some(vec![StepItem {
                data: Some(StepData::Evm(EvmStepData {
                    to: to.to_string(),
                    data: Some(data.to_string()),
                    value: value.to_string(),
                })),
            }]),
        }
    }

    pub fn mock_empty(id: &str, kind: &str) -> Self {
        Self {
            id: id.to_string(),
            kind: kind.to_string(),
            items: None,
        }
    }
}
