// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::execution_status::{CommandIndex, ExecutionFailureStatus};
use crate::{ObjectID, SequenceNumber};
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

    #[error("Invalid address")]
    InvalidAddress,
    #[error("Invalid transaction digest.")]
    InvalidTransactionDigest,
    #[error("Invalid digest length. Expected {expected}, got {actual}")]
    InvalidDigestLength { expected: usize, actual: usize },

    #[error("Failure deserializing object in the requested format: {:?}", error)]
    ObjectDeserializationError { error: String },
}

#[derive(
    Eq, PartialEq, Clone, Debug, Serialize, Deserialize, Error, Hash, AsRefStr, IntoStaticStr,
)]
pub enum UserInputError {
    #[error("Size limit exceeded: {limit} is {value}")]
    SizeLimitExceeded { limit: String, value: String },
    #[error("This Move function is currently disabled and not available for call")]
    BlockedMoveFunction,
    #[error(
        "TransferObjects, MergeCoin, and Publish cannot have empty arguments. \
        If MakeMoveVec has empty arguments, it must have a type specified"
    )]
    EmptyCommandInput,
    #[error("The transaction inputs contain duplicated ObjectRef's")]
    DuplicateObjectRefInput,
    #[error(
        "{max_publish_commands} max publish/upgrade commands allowed, {publish_count} provided"
    )]
    MaxPublishCountExceeded {
        max_publish_commands: u64,
        publish_count: u64,
    },
    #[error("Commands following a command with Random can only be TransferObjects or MergeCoins")]
    PostRandomCommandRestrictions,
    #[error("Dependent package not found on-chain: {package_id:?}")]
    DependentPackageNotFound { package_id: ObjectID },
    #[error(
        "Could not find the referenced object {:?} at version {:?}",
        object_id,
        version
    )]
    ObjectNotFound {
        object_id: ObjectID,
        version: Option<SequenceNumber>,
    },
}
