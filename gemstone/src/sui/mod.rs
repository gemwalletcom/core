pub mod gas_budget;
mod model;

use crate::GemstoneError;
use gem_sui::models::{StakeInput, TokenTransferInput, TransferInput, UnstakeInput};
use model::{SuiStakeInput, SuiTokenTransferInput, SuiTransferInput, SuiTxOutput, SuiUnstakeInput};

/// Sui
#[uniffi::export]
pub fn sui_encode_transfer(input: &SuiTransferInput) -> Result<SuiTxOutput, GemstoneError> {
    let inner: TransferInput = input.into();
    gem_sui::encode_transfer(&inner).map(SuiTxOutput::from).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_token_transfer(input: &SuiTokenTransferInput) -> Result<SuiTxOutput, GemstoneError> {
    let inner: TokenTransferInput = input.into();
    gem_sui::encode_token_transfer(&inner).map(SuiTxOutput::from).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_split_stake(input: &SuiStakeInput) -> Result<SuiTxOutput, GemstoneError> {
    let inner: StakeInput = input.into();
    gem_sui::encode_split_and_stake(&inner).map(SuiTxOutput::from).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_encode_unstake(input: &SuiUnstakeInput) -> Result<SuiTxOutput, GemstoneError> {
    let inner: UnstakeInput = input.into();
    gem_sui::encode_unstake(&inner).map(SuiTxOutput::from).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn sui_validate_and_hash(encoded: String) -> Result<SuiTxOutput, GemstoneError> {
    gem_sui::tx::validate_and_hash(&encoded).map(SuiTxOutput::from).map_err(GemstoneError::from)
}
