use crate::SwapperError;
use gem_sui::SuiError;
use std::fmt::Display;

pub(super) fn tx_error(error: impl Display) -> SwapperError {
    SwapperError::TransactionError(error.to_string())
}

pub(super) fn sui_error(error: SuiError) -> SwapperError {
    SwapperError::TransactionError(error.to_string())
}
