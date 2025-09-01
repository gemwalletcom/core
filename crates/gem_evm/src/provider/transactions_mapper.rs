use primitives::{TransactionUpdate, BroadcastOptions};

pub fn map_transaction_broadcast(_data: String, _options: BroadcastOptions) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    unimplemented!("map_transaction_broadcast")
}

pub fn map_transaction_status(_transaction_data: String) -> Result<TransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
    unimplemented!("map_transaction_status")
}