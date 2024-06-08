// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::base_types::ObjectRef;
use crate::digests::{ObjectDigest, TransactionDigest};
use crate::execution_status::{CommandIndex, ExecutionFailureStatus};
use crate::governance::EpochId;
use crate::{ObjectID, SequenceNumber, SuiAddress};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, IntoStaticStr};
use thiserror::Error;

pub const TRANSACTION_NOT_FOUND_MSG_PREFIX: &str = "Could not find the referenced transaction";
pub const TRANSACTIONS_NOT_FOUND_MSG_PREFIX: &str = "Could not find the referenced transactions";

#[macro_export]
macro_rules! fp_bail {
    ($e:expr) => {
        return Err($e)
    };
}

#[macro_export(local_inner_macros)]
macro_rules! fp_ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            fp_bail!($e);
        }
    };
}
pub(crate) use fp_ensure;

pub type SuiResult<T = ()> = Result<T, SuiError>;
pub type UserInputResult<T = ()> = Result<T, UserInputError>;

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type ExecutionErrorKind = ExecutionFailureStatus;

#[derive(Debug)]
pub struct ExecutionError {
    inner: Box<ExecutionErrorInner>,
}

#[derive(Debug)]
struct ExecutionErrorInner {
    kind: ExecutionErrorKind,
    source: Option<BoxError>,
    command: Option<CommandIndex>,
}

impl ExecutionError {
    pub fn new(kind: ExecutionErrorKind, source: Option<BoxError>) -> Self {
        Self {
            inner: Box::new(ExecutionErrorInner {
                kind,
                source,
                command: None,
            }),
        }
    }

    pub fn new_with_source<E: Into<BoxError>>(kind: ExecutionErrorKind, source: E) -> Self {
        Self::new(kind, Some(source.into()))
    }

    pub fn with_command_index(mut self, command: CommandIndex) -> Self {
        self.inner.command = Some(command);
        self
    }

    pub fn from_kind(kind: ExecutionErrorKind) -> Self {
        Self::new(kind, None)
    }

    pub fn kind(&self) -> &ExecutionErrorKind {
        &self.inner.kind
    }

    pub fn command(&self) -> Option<CommandIndex> {
        self.inner.command
    }

    pub fn source(&self) -> &Option<BoxError> {
        &self.inner.source
    }

    pub fn to_execution_status(&self) -> (ExecutionFailureStatus, Option<CommandIndex>) {
        (self.kind().clone(), self.command())
    }
}

/// Custom error type for Sui.
#[derive(
    Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash, AsRefStr, IntoStaticStr,
)]
pub enum SuiError {
    #[error("Error checking transaction input objects: {:?}", error)]
    UserInputError { error: UserInputError },

    #[error("Expecting a single owner, shared ownership found")]
    UnexpectedOwnerType,

    #[error("There are already {queue_len} transactions pending, above threshold of {threshold}")]
    TooManyTransactionsPendingExecution { queue_len: usize, threshold: usize },

    #[error("There are too many transactions pending in consensus")]
    TooManyTransactionsPendingConsensus,

    #[error("Input {object_id} already has {queue_len} transactions pending, above threshold of {threshold}")]
    TooManyTransactionsPendingOnObject {
        object_id: ObjectID,
        queue_len: usize,
        threshold: usize,
    },

    #[error("Input {object_id} has a transaction {txn_age_sec} seconds old pending, above threshold of {threshold} seconds")]
    TooOldTransactionPendingOnObject {
        object_id: ObjectID,
        txn_age_sec: u64,
        threshold: u64,
    },

    #[error("Soft bundle must only contain transactions of UserTransaction kind")]
    InvalidTxKindInSoftBundle,

    // Signature verification
    #[error("Signature is not valid: {}", error)]
    InvalidSignature { error: String },
    #[error("Required Signature from {expected} is absent {:?}", actual)]
    SignerSignatureAbsent {
        expected: String,
        actual: Vec<String>,
    },
    #[error("Expect {expected} signer signatures but got {actual}")]
    SignerSignatureNumberMismatch { expected: usize, actual: usize },
    #[error("Value was not signed by the correct sender: {}", error)]
    IncorrectSigner { error: String },
    // TODO: Used for distinguishing between different occurrences of invalid signatures, to allow retries in some cases.
    #[error(
        "Signature is not valid, but a retry may result in a valid one: {}",
        error
    )]
    PotentiallyTemporarilyInvalidSignature { error: String },

    // Certificate verification and execution
    #[error(
        "Signature or certificate from wrong epoch, expected {expected_epoch}, got {actual_epoch}"
    )]
    WrongEpoch {
        expected_epoch: EpochId,
        actual_epoch: EpochId,
    },
    #[error("Signatures in a certificate must form a quorum")]
    CertificateRequiresQuorum,
    #[error("Transaction certificate processing failed: {err}")]
    ErrorWhileProcessingCertificate { err: String },

    #[error("Transaction is already finalized but with different user signatures")]
    TxAlreadyFinalizedWithDifferentUserSigs,

    // Account access
    #[error("Invalid authenticator")]
    InvalidAuthenticator,
    #[error("Invalid address")]
    InvalidAddress,
    #[error("Invalid transaction digest.")]
    InvalidTransactionDigest,

    #[error("Invalid digest length. Expected {expected}, got {actual}")]
    InvalidDigestLength { expected: usize, actual: usize },

    #[error("Unexpected message.")]
    UnexpectedMessage,

    // Move module publishing related errors
    #[error("Failed to verify the Move module, reason: {error:?}.")]
    ModuleVerificationFailure { error: String },
    #[error("Failed to deserialize the Move module, reason: {error:?}.")]
    ModuleDeserializationFailure { error: String },
    #[error("Failed to publish the Move module(s), reason: {error}")]
    ModulePublishFailure { error: String },
    #[error("Failed to build Move modules: {error}.")]
    ModuleBuildFailure { error: String },

    // Move call related errors
    #[error("Function resolution failure: {error:?}.")]
    FunctionNotFound { error: String },
    #[error("Module not found in package: {module_name:?}.")]
    ModuleNotFound { module_name: String },
    #[error("Type error while binding function arguments: {error:?}.")]
    TypeError { error: String },
    #[error("Circular object ownership detected")]
    CircularObjectOwnership,

    // Internal state errors
    #[error("Attempt to re-initialize a transaction lock for objects {:?}.", refs)]
    ObjectLockAlreadyInitialized { refs: Vec<ObjectRef> },
    #[error(
        "Object {obj_ref:?} already locked by a different transaction: {pending_transaction:?}"
    )]
    ObjectLockConflict {
        obj_ref: ObjectRef,
        pending_transaction: TransactionDigest,
    },
    #[error("Objects {obj_refs:?} are already locked by a transaction from a future epoch {locked_epoch:?}), attempt to override with a transaction from epoch {new_epoch:?}")]
    ObjectLockedAtFutureEpoch {
        obj_refs: Vec<ObjectRef>,
        locked_epoch: EpochId,
        new_epoch: EpochId,
        locked_by_tx: TransactionDigest,
    },
    #[error("{TRANSACTION_NOT_FOUND_MSG_PREFIX} [{:?}].", digest)]
    TransactionNotFound { digest: TransactionDigest },
    #[error("{TRANSACTIONS_NOT_FOUND_MSG_PREFIX} [{:?}].", digests)]
    TransactionsNotFound { digests: Vec<TransactionDigest> },
    #[error(
        "Attempt to move to `Executed` state an transaction that has already been executed: {:?}.",
        digest
    )]
    TransactionAlreadyExecuted { digest: TransactionDigest },
    #[error("Object ID did not have the expected type")]
    BadObjectType { error: String },
    #[error("Fail to retrieve Object layout for {st}")]
    FailObjectLayout { st: String },

    #[error("Execution invariant violated")]
    ExecutionInvariantViolation,
    #[allow(non_camel_case_types)]
    #[serde(rename = "StorageError")]
    #[error("DEPRECATED")]
    DEPRECATED_StorageError,
    #[allow(non_camel_case_types)]
    #[serde(rename = "GenericStorageError")]
    #[error("DEPRECATED")]
    DEPRECATED_GenericStorageError,

    #[allow(non_camel_case_types)]
    #[serde(rename = "StorageMissingFieldError")]
    #[error("DEPRECATED")]
    DEPRECATED_StorageMissingFieldError,
    #[allow(non_camel_case_types)]
    #[serde(rename = "StorageCorruptedFieldError")]
    #[error("DEPRECATED")]
    DEPRECATED_StorageCorruptedFieldError,

    #[error("Authority Error: {error:?}")]
    GenericAuthorityError { error: String },

    #[error("Generic Bridge Error: {error:?}")]
    GenericBridgeError { error: String },

    #[error("Failed to dispatch subscription: {error:?}")]
    FailedToDispatchSubscription { error: String },

    #[error("Failed to serialize Owner: {error:?}")]
    OwnerFailedToSerialize { error: String },

    #[error("Failed to deserialize fields into JSON: {error:?}")]
    ExtraFieldFailedToDeserialize { error: String },

    #[error("Failed to execute transaction locally by Orchestrator: {error:?}")]
    TransactionOrchestratorLocalExecutionError { error: String },

    // Errors returned by authority and client read API's
    #[error("Failure serializing transaction in the requested format: {:?}", error)]
    TransactionSerializationError { error: String },
    #[error("Failure serializing object in the requested format: {:?}", error)]
    ObjectSerializationError { error: String },
    #[error("Failure deserializing object in the requested format: {:?}", error)]
    ObjectDeserializationError { error: String },
    #[error("Event store component is not active on this node")]
    NoEventStore,

    // Client side error
    #[error("Invalid transaction range query to the fullnode: {:?}", error)]
    FullNodeInvalidTxRangeQuery { error: String },

    // Errors related to the authority-consensus interface.
    #[error("Failed to submit transaction to consensus: {0}")]
    FailedToSubmitToConsensus(String),
    #[error("Failed to connect with consensus node: {0}")]
    ConsensusConnectionBroken(String),
    #[error("Failed to execute handle_consensus_transaction on Sui: {0}")]
    HandleConsensusTransactionFailure(String),

    // Cryptography errors.
    #[error("Signature key generation error: {0}")]
    SignatureKeyGenError(String),
    #[error("Key Conversion Error: {0}")]
    KeyConversionError(String),
    #[error("Invalid Private Key provided")]
    InvalidPrivateKey,

    // Unsupported Operations on Fullnode
    #[error("Fullnode does not support handle_certificate")]
    FullNodeCantHandleCertificate,

    // Epoch related errors.
    #[error("Validator temporarily stopped processing transactions due to epoch change")]
    ValidatorHaltedAtEpochEnd,
    #[error("Operations for epoch {0} have ended")]
    EpochEnded(EpochId),
    #[error("Error when advancing epoch: {:?}", error)]
    AdvanceEpochError { error: String },

    #[error("Transaction Expired")]
    TransactionExpired,

    // These are errors that occur when an RPC fails and is simply the utf8 message sent in a
    // Tonic::Status
    #[error("{1} - {0}")]
    RpcError(String, String),

    #[error("Method not allowed")]
    InvalidRpcMethodError,

    #[error("Use of disabled feature: {:?}", error)]
    UnsupportedFeatureError { error: String },

    #[error("Unable to communicate with the Quorum Driver channel: {:?}", error)]
    QuorumDriverCommunicationError { error: String },

    #[error("Operation timed out")]
    TimeoutError,

    #[error("Error executing {0}")]
    ExecutionError(String),

    #[error("Invalid committee composition")]
    InvalidCommittee(String),

    #[error("Missing committee information for epoch {0}")]
    MissingCommitteeAtEpoch(EpochId),

    #[error("Index store not available on this Fullnode.")]
    IndexStoreNotAvailable,

    #[error("Failed to read dynamic field from table in the object store: {0}")]
    DynamicFieldReadError(String),

    #[error("Failed to read or deserialize system state related data structures on-chain: {0}")]
    SuiSystemStateReadError(String),

    #[error("Failed to read or deserialize bridge related data structures on-chain: {0}")]
    SuiBridgeReadError(String),

    #[error("Unexpected version error: {0}")]
    UnexpectedVersion(String),

    #[error("Message version is not supported at the current protocol version: {error}")]
    WrongMessageVersion { error: String },

    #[error("unknown error: {0}")]
    Unknown(String),

    #[error("Failed to perform file operation: {0}")]
    FileIOError(String),

    #[error("Failed to get JWK")]
    JWKRetrievalError,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Validator cannot handle the request at the moment. Please retry after at least {retry_after_secs} seconds.")]
    ValidatorOverloadedRetryAfter { retry_after_secs: u64 },

    #[error("Too many requests")]
    TooManyRequests,
}

#[derive(
    Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash, AsRefStr, IntoStaticStr,
)]
pub enum UserInputError {
    #[error("Mutable object {object_id} cannot appear more than one in one transaction")]
    MutableObjectUsedMoreThanOnce { object_id: ObjectID },
    #[error("Wrong number of parameters for the transaction")]
    ObjectInputArityViolation,
    #[error(
        "Could not find the referenced object {:?} at version {:?}",
        object_id,
        version
    )]
    ObjectNotFound {
        object_id: ObjectID,
        version: Option<SequenceNumber>,
    },
    #[error("Object {provided_obj_ref:?} is not available for consumption, its current version: {current_version:?}")]
    ObjectVersionUnavailableForConsumption {
        provided_obj_ref: ObjectRef,
        current_version: SequenceNumber,
    },
    #[error("Package verification failed: {err:?}")]
    PackageVerificationTimedout { err: String },
    #[error("Dependent package not found on-chain: {package_id:?}")]
    DependentPackageNotFound { package_id: ObjectID },
    #[error("Mutable parameter provided, immutable parameter expected")]
    ImmutableParameterExpectedError { object_id: ObjectID },
    #[error("Size limit exceeded: {limit} is {value}")]
    SizeLimitExceeded { limit: String, value: String },
    #[error(
        "Object {child_id:?} is owned by object {parent_id:?}. \
        Objects owned by other objects cannot be used as input arguments"
    )]
    InvalidChildObjectArgument {
        child_id: ObjectID,
        parent_id: ObjectID,
    },
    #[error(
        "Invalid Object digest for object {object_id:?}. Expected digest : {expected_digest:?}"
    )]
    InvalidObjectDigest {
        object_id: ObjectID,
        expected_digest: ObjectDigest,
    },
    #[error("Sequence numbers above the maximal value are not usable for transfers")]
    InvalidSequenceNumber,
    #[error("A move object is expected, instead a move package is passed: {object_id}")]
    MovePackageAsObject { object_id: ObjectID },
    #[error("A move package is expected, instead a move object is passed: {object_id}")]
    MoveObjectAsPackage { object_id: ObjectID },
    #[error("Transaction was not signed by the correct sender: {}", error)]
    IncorrectUserSignature { error: String },

    #[error("Object used as shared is not shared")]
    NotSharedObjectError,
    #[error("The transaction inputs contain duplicated ObjectRef's")]
    DuplicateObjectRefInput,

    // Gas related errors
    #[error("Transaction gas payment missing")]
    MissingGasPayment,
    // #[error("Gas object is not an owned object with owner: {:?}", owner)]
    // GasObjectNotOwnedObject { owner: Owner },
    #[error("Gas budget: {:?} is higher than max: {:?}", gas_budget, max_budget)]
    GasBudgetTooHigh { gas_budget: u64, max_budget: u64 },
    #[error("Gas budget: {:?} is lower than min: {:?}", gas_budget, min_budget)]
    GasBudgetTooLow { gas_budget: u64, min_budget: u64 },
    #[error(
        "Balance of gas object {:?} is lower than the needed amount: {:?}",
        gas_balance,
        needed_gas_amount
    )]
    GasBalanceTooLow {
        gas_balance: u128,
        needed_gas_amount: u128,
    },
    #[error("Transaction kind does not support Sponsored Transaction")]
    UnsupportedSponsoredTransactionKind,
    #[error(
        "Gas price {:?} under reference gas price (RGP) {:?}",
        gas_price,
        reference_gas_price
    )]
    GasPriceUnderRGP {
        gas_price: u64,
        reference_gas_price: u64,
    },
    #[error("Gas price cannot exceed {:?} mist", max_gas_price)]
    GasPriceTooHigh { max_gas_price: u64 },
    #[error("Object {object_id} is not a gas object")]
    InvalidGasObject { object_id: ObjectID },
    #[error("Gas object does not have enough balance to cover minimal gas spend")]
    InsufficientBalanceToCoverMinimalGas,

    #[error("Could not find the referenced object {:?} as the asked version {:?} is higher than the latest {:?}", object_id, asked_version, latest_version)]
    ObjectSequenceNumberTooHigh {
        object_id: ObjectID,
        asked_version: SequenceNumber,
        latest_version: SequenceNumber,
    },
    #[error("Object deleted at reference {:?}", object_ref)]
    ObjectDeleted { object_ref: ObjectRef },
    #[error("Invalid Batch Transaction: {}", error)]
    InvalidBatchTransaction { error: String },
    #[error("This Move function is currently disabled and not available for call")]
    BlockedMoveFunction,
    #[error("Empty input coins for Pay related transaction")]
    EmptyInputCoins,

    #[error("SUI payment transactions use first input coin for gas payment, but found a different gas object")]
    UnexpectedGasPaymentObject,

    #[error("Wrong initial version given for shared object")]
    SharedObjectStartingVersionMismatch,

    #[error("Attempt to transfer object {object_id} that does not have public transfer. Object transfer must be done instead using a distinct Move function call")]
    TransferObjectWithoutPublicTransferError { object_id: ObjectID },

    #[error(
        "TransferObjects, MergeCoin, and Publish cannot have empty arguments. \
        If MakeMoveVec has empty arguments, it must have a type specified"
    )]
    EmptyCommandInput,

    #[error("Transaction is denied: {}", error)]
    TransactionDenied { error: String },

    #[error("Feature is not supported: {0}")]
    Unsupported(String),

    #[error("Query transactions with move function input error: {0}")]
    MoveFunctionInputError(String),

    #[error("Verified checkpoint not found for digest: {0}")]
    VerifiedCheckpointDigestNotFound(String),

    #[error("Latest checkpoint sequence number not found")]
    LatestCheckpointSequenceNumberNotFound,

    #[error("Genesis transaction not found")]
    GenesisTransactionNotFound,

    #[error("Transaction {0} not found")]
    TransactionCursorNotFound(u64),

    #[error(
        "Object {:?} is a system object and cannot be accessed by user transactions",
        object_id
    )]
    InaccessibleSystemObject { object_id: ObjectID },
    #[error(
        "{max_publish_commands} max publish/upgrade commands allowed, {publish_count} provided"
    )]
    MaxPublishCountExceeded {
        max_publish_commands: u64,
        publish_count: u64,
    },

    #[error("Immutable parameter provided, mutable parameter expected")]
    MutableParameterExpected { object_id: ObjectID },

    #[error("Address {address:?} is denied for coin {coin_type}")]
    AddressDeniedForCoin {
        address: SuiAddress,
        coin_type: String,
    },

    #[error("Commands following a command with Random can only be TransferObjects or MergeCoins")]
    PostRandomCommandRestrictions,
}
