use crate::models::transaction::{Input, Output, Transaction};

impl Transaction {
    pub fn mock() -> Self {
        Self {
            txid: "abc123".to_string(),
            value: "100000".to_string(),
            value_in: "105000".to_string(),
            fees: "5000".to_string(),
            block_time: 1640995200,
            block_height: 700000,
            vin: vec![],
            vout: vec![],
        }
    }
}

impl Input {
    pub fn mock() -> Self {
        Self {
            is_address: true,
            addresses: Some(vec!["bc1qinput".to_string()]),
            value: "105000".to_string(),
            n: 0,
            tx_id: Some("prev_tx".to_string()),
            vout: Some(0),
        }
    }
}

impl Output {
    pub fn mock() -> Self {
        Self {
            is_address: true,
            addresses: Some(vec!["bc1qoutput".to_string()]),
            value: "100000".to_string(),
            n: 0,
        }
    }
}
