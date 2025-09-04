use crate::{AssetId, Chain, Transaction, TransactionState, TransactionType};
use chrono::Utc;

impl Transaction {
    pub fn mock() -> Self {
        Transaction::new(
            "0x1234567890abcdef".to_string(),
            AssetId::from_chain(Chain::Ethereum),
            "0xfrom".to_string(),
            "0xto".to_string(),
            None,
            TransactionType::Transfer,
            TransactionState::Confirmed,
            "21000".to_string(),
            AssetId::from_chain(Chain::Ethereum),
            "1000000".to_string(),
            None,
            None,
            Utc::now(),
        )
    }

    pub fn mock_with_params(
        asset_id: AssetId,
        transaction_type: TransactionType,
        value: String,
    ) -> Self {
        Transaction::new(
            "0x1234567890abcdef".to_string(),
            asset_id.clone(),
            "0xfrom".to_string(),
            "0xto".to_string(),
            None,
            transaction_type,
            TransactionState::Confirmed,
            "21000".to_string(),
            asset_id,
            value,
            None,
            None,
            Utc::now(),
        )
    }
}