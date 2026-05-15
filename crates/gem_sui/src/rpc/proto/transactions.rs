use serde_json::Value;

use super::json::decode_json_value;
use super::{Bcs, FieldMask, Owner, Status, Timestamp, Transaction, UserSignature};
use gem_encoding::protobuf::{proto_decode, proto_encode};

// Field numbers mirror sui-rpc v0.3.1 transaction response schemas:
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/ledger_service.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/transaction_execution_service.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/executed_transaction.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/effects.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/event.proto
// https://docs.rs/crate/sui-rpc/0.3.1/source/vendored/proto/sui/rpc/v2/balance_change.proto

#[derive(Clone, Debug, Default)]
pub struct GetTransactionRequest {
    pub digest: Option<String>,
    pub read_mask: Option<FieldMask>,
}

proto_encode!(GetTransactionRequest {
    1 => digest: optional_string,
    2 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct GetTransactionResponse {
    pub transaction: Option<ExecutedTransaction>,
}

proto_decode!(GetTransactionResponse {
    1 => transaction: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct BatchGetTransactionsRequest {
    pub digests: Vec<String>,
    pub read_mask: Option<FieldMask>,
}

proto_encode!(BatchGetTransactionsRequest {
    1 => digests: repeated_string,
    2 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct BatchGetTransactionsResponse {
    pub transactions: Vec<GetTransactionResult>,
}

proto_decode!(BatchGetTransactionsResponse {
    1 => transactions: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub enum GetTransactionResult {
    Transaction(Box<ExecutedTransaction>),
    Error(Status),
    #[default]
    Unknown,
}

proto_decode!(GetTransactionResult {
    1 => |value, field| *value = Self::Transaction(Box::new(field.message()?)),
    2 => |value, field| *value = Self::Error(field.message()?),
});

#[derive(Clone, Debug, Default)]
pub struct ExecuteTransactionRequest {
    pub transaction: Option<Transaction>,
    pub signatures: Vec<UserSignature>,
    pub read_mask: Option<FieldMask>,
}

proto_encode!(ExecuteTransactionRequest {
    1 => transaction: optional_message,
    2 => signatures: repeated_message,
    3 => read_mask: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ExecuteTransactionResponse {
    pub transaction: Option<ExecutedTransaction>,
}

proto_decode!(ExecuteTransactionResponse {
    1 => transaction: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct SimulateTransactionRequest {
    pub transaction: Option<Transaction>,
    pub read_mask: Option<FieldMask>,
    pub checks: Option<TransactionChecks>,
}

impl SimulateTransactionRequest {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction: Some(transaction),
            read_mask: None,
            checks: None,
        }
    }
}

proto_encode!(SimulateTransactionRequest {
    1 => transaction: optional_message,
    2 => read_mask: optional_message,
    3 => checks: optional_enum_varint,
});

#[derive(Clone, Copy, Debug)]
pub enum TransactionChecks {
    Enabled = 0,
    Disabled = 1,
}

impl From<TransactionChecks> for u64 {
    fn from(value: TransactionChecks) -> Self {
        value as u64
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimulateTransactionResponse {
    pub transaction: Option<ExecutedTransaction>,
    pub command_outputs: Vec<CommandResult>,
}

proto_decode!(SimulateTransactionResponse {
    1 => transaction: optional_message,
    2 => command_outputs: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct CommandResult {
    pub return_values: Vec<CommandOutput>,
}

proto_decode!(CommandResult {
    1 => return_values: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct CommandOutput {
    pub value: Option<Bcs>,
}

proto_decode!(CommandOutput {
    2 => value: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ExecutedTransaction {
    pub digest: Option<String>,
    pub effects: Option<TransactionEffects>,
    pub events: Option<TransactionEvents>,
    pub timestamp: Option<Timestamp>,
    pub balance_changes: Vec<BalanceChange>,
}

proto_decode!(ExecutedTransaction {
    1 => digest: optional_string,
    4 => effects: optional_message,
    5 => events: optional_message,
    7 => timestamp: optional_message,
    8 => balance_changes: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct TransactionEffects {
    pub status: Option<ExecutionStatus>,
    pub gas_used: Option<GasCostSummary>,
    pub gas_object: Option<ChangedObject>,
}

proto_decode!(TransactionEffects {
    4 => status: optional_message,
    6 => gas_used: optional_message,
    8 => gas_object: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ExecutionStatus {
    pub success: Option<bool>,
    pub error: Option<ExecutionError>,
}

proto_decode!(ExecutionStatus {
    1 => success: optional_bool,
    2 => error: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct ExecutionError {
    pub description: Option<String>,
}

proto_decode!(ExecutionError {
    1 => description: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct GasCostSummary {
    pub computation_cost: Option<u64>,
    pub storage_cost: Option<u64>,
    pub storage_rebate: Option<u64>,
    pub non_refundable_storage_fee: Option<u64>,
}

proto_decode!(GasCostSummary {
    1 => computation_cost: optional_varint_u64,
    2 => storage_cost: optional_varint_u64,
    3 => storage_rebate: optional_varint_u64,
    4 => non_refundable_storage_fee: optional_varint_u64,
});

#[derive(Clone, Debug, Default)]
pub struct ChangedObject {
    pub input_owner: Option<Owner>,
    pub output_owner: Option<Owner>,
}

proto_decode!(ChangedObject {
    5 => input_owner: optional_message,
    9 => output_owner: optional_message,
});

#[derive(Clone, Debug, Default)]
pub struct BalanceChange {
    pub address: Option<String>,
    pub coin_type: Option<String>,
    pub amount: Option<String>,
}

proto_decode!(BalanceChange {
    1 => address: optional_string,
    2 => coin_type: optional_string,
    3 => amount: optional_string,
});

#[derive(Clone, Debug, Default)]
pub struct TransactionEvents {
    pub events: Vec<Event>,
}

proto_decode!(TransactionEvents {
    3 => events: repeated_message,
});

#[derive(Clone, Debug, Default)]
pub struct Event {
    pub package_id: Option<String>,
    pub event_type: Option<String>,
    pub json: Option<Value>,
}

proto_decode!(Event {
    1 => |value, field| value.package_id = Some(field.string()?),
    4 => |value, field| value.event_type = Some(field.string()?),
    6 => |value, field| value.json = Some(decode_json_value(field.bytes()?)?),
});

#[cfg(test)]
mod tests {
    use super::*;
    use gem_encoding::protobuf::{MessageDecode, encode_message_field, encode_string_field};

    #[test]
    fn test_execute_transaction_response_decode() {
        let digest = "HgFLiBHYjYKhEh5NPY52Zm9bhwrs4W6AxE46gMU7nwhZ";
        let transaction = encode_string_field(1, digest);
        let response = encode_message_field(1, &transaction);

        assert_eq!(ExecuteTransactionResponse::decode(&response).unwrap().transaction.unwrap().digest.unwrap(), digest);
    }
}
