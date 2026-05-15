pub(crate) mod balances;
pub(crate) mod bcs;
pub(crate) mod checkpoints;
pub(crate) mod field;
pub(crate) mod json;
pub(crate) mod objects;
pub(crate) mod service;
pub(crate) mod status;
pub(crate) mod timestamp;
pub(crate) mod transaction_data;
pub(crate) mod transactions;

pub(crate) use gem_encoding::protobuf::MessageResult;

pub(crate) use balances::{GetBalanceRequest, GetBalanceResponse, GetCoinInfoRequest, GetCoinInfoResponse, ListBalancesRequest, ListBalancesResponse};
pub(crate) use bcs::Bcs;
pub(crate) use checkpoints::{Checkpoint, GetCheckpointRequest, GetCheckpointResponse};
pub(crate) use field::FieldMask;
pub(crate) use objects::{
    BatchGetObjectsRequest, BatchGetObjectsResponse, GetObjectRequest, GetObjectResponse, GetObjectResult, ListOwnedObjectsRequest, ListOwnedObjectsResponse, Object, Owner,
    OwnerKind,
};
pub(crate) use service::{Epoch, GetEpochRequest, GetEpochResponse, GetServiceInfoRequest, GetServiceInfoResponse};
pub(crate) use status::Status;
pub(crate) use timestamp::Timestamp;
pub(crate) use transaction_data::{Argument, Input, MoveCall, ProgrammableTransaction, Transaction, TransactionKind, UserSignature};
pub(crate) use transactions::{
    BalanceChange, BatchGetTransactionsRequest, BatchGetTransactionsResponse, ExecuteTransactionRequest, ExecuteTransactionResponse, ExecutedTransaction, GasCostSummary,
    GetTransactionRequest, GetTransactionResponse, GetTransactionResult, SimulateTransactionRequest, SimulateTransactionResponse, TransactionChecks, TransactionEffects,
    TransactionEvents,
};

pub(crate) trait WithMut: Sized {
    fn with(mut self, update: impl FnOnce(&mut Self)) -> Self {
        update(&mut self);
        self
    }
}

impl<T> WithMut for T {}
