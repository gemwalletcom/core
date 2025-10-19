mod model;

use crate::{
    GemstoneError,
    alien::{AlienSigner, SigningAlgorithm},
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ed25519_dalek::SigningKey;
use gem_sui::models::{StakeInput, TokenTransferInput, TransferInput, UnstakeInput};
use model::{SuiStakeInput, SuiTokenTransferInput, SuiTransferInput, SuiTxOutput, SuiUnstakeInput};
use std::{borrow::Cow, sync::Arc};
use sui_types::PersonalMessage;

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

#[uniffi::export]
pub fn sui_sign_personal_message(signer: Arc<dyn AlienSigner>, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, crate::alien::AlienError> {
    let personal_message = PersonalMessage(Cow::Owned(message));
    let digest = personal_message.signing_digest();
    sui_sign_digest(signer, digest.to_vec(), private_key)
}

#[uniffi::export]
pub fn sui_sign_digest(signer: Arc<dyn AlienSigner>, digest: Vec<u8>, private_key: Vec<u8>) -> Result<String, crate::alien::AlienError> {
    let signature = signer.sign(digest, SigningAlgorithm::Ed25519, private_key.clone())?;
    assemble_sui_signature(signature, private_key)
}

fn assemble_sui_signature(signature: Vec<u8>, private_key: Vec<u8>) -> Result<String, crate::alien::AlienError> {
    if signature.len() != ed25519_dalek::Signature::BYTE_SIZE {
        return Err(crate::alien::AlienError::SigningError {
            msg: format!(
                "Expected {} byte ed25519 signature, got {}",
                ed25519_dalek::Signature::BYTE_SIZE,
                signature.len()
            ),
        });
    }

    let key_bytes: [u8; ed25519_dalek::SECRET_KEY_LENGTH] = private_key.as_slice().try_into().map_err(|_| crate::alien::AlienError::SigningError {
        msg: "Invalid Ed25519 private key length".to_string(),
    })?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let public_key = signing_key.verifying_key().to_bytes();

    let mut sui_bytes = Vec::with_capacity(1 + signature.len() + public_key.len());
    sui_bytes.push(0x00);
    sui_bytes.extend_from_slice(&signature);
    sui_bytes.extend_from_slice(&public_key);
    Ok(BASE64.encode(sui_bytes))
}
