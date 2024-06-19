// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use thiserror::Error;

pub type FastCryptoResult<T> = Result<T, FastCryptoError>;

/// Collection of errors to be used in fastcrypto.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum FastCryptoError {
    /// Invalid value was given to the function
    #[error("Invalid value was given to the function")]
    InvalidInput,
}
