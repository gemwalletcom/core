use super::model::{RelayCurrencyDetail, RelayRequest, RelayRequestMetadata, RelayStatus, Step, StepData, StepItem};

impl RelayRequest {
    pub fn mock(status: RelayStatus, metadata: Option<RelayRequestMetadata>) -> Self {
        Self { status, metadata }
    }
}

impl RelayCurrencyDetail {
    pub fn mock(currency: &str, chain_id: u64, amount: &str) -> Self {
        Self {
            currency: currency.to_string(),
            chain_id,
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
                data: Some(StepData {
                    to: Some(to.to_string()),
                    data: Some(data.to_string()),
                    value: value.to_string(),
                    instructions: None,
                    address_lookup_table_addresses: None,
                    psbt: None,
                }),
            }]),
        }
    }

    pub fn mock_bitcoin(psbt: &str) -> Self {
        Self {
            id: "deposit".to_string(),
            kind: "transaction".to_string(),
            items: Some(vec![StepItem {
                data: Some(StepData {
                    to: None,
                    data: None,
                    value: String::new(),
                    instructions: None,
                    address_lookup_table_addresses: None,
                    psbt: Some(psbt.to_string()),
                }),
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
