use super::model::{RelayCurrencyDetail, RelayRequest, RelayRequestMetadata, RelayStatus, Step, StepData, StepItem};

pub fn create_relay_request(status: RelayStatus, metadata: Option<RelayRequestMetadata>) -> RelayRequest {
    RelayRequest { status, metadata }
}

pub fn create_currency_detail(currency: &str, chain_id: u64, amount: &str) -> RelayCurrencyDetail {
    RelayCurrencyDetail {
        currency: currency.to_string(),
        chain_id,
        amount: Some(amount.to_string()),
    }
}

pub fn create_transaction_step(id: &str, to: &str, value: &str, data: &str) -> Step {
    Step {
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

pub fn create_bitcoin_step(psbt: &str) -> Step {
    Step {
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

pub fn create_empty_step(id: &str, kind: &str) -> Step {
    Step {
        id: id.to_string(),
        kind: kind.to_string(),
        items: None,
    }
}
