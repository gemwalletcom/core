use crate::{TransferDataExtra, TransferDataOutputAction, TransferDataOutputType};

impl TransferDataExtra {
    pub fn mock() -> Self {
        TransferDataExtra {
            to: "".to_string(),
            gas_limit: None,
            gas_price: None,
            data: None,
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Sign,
        }
    }

    pub fn mock_encoded_transaction(data: Vec<u8>) -> Self {
        TransferDataExtra {
            to: "".to_string(),
            gas_limit: None,
            gas_price: None,
            data: Some(data),
            output_type: TransferDataOutputType::EncodedTransaction,
            output_action: TransferDataOutputAction::Sign,
        }
    }
}
