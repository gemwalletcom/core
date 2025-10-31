use crate::{SignerError, TransactionLoadInput};

pub trait ChainSigner: Send + Sync {
    fn sign_transfer(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_transfer not implemented".to_string()))
    }

    fn sign_token_transfer(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_token_transfer not implemented".to_string()))
    }

    fn sign_nft_transfer(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_nft_transfer not implemented".to_string()))
    }

    fn sign_swap(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_swap not implemented".to_string()))
    }

    fn sign_token_approval(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_token_approval not implemented".to_string()))
    }

    fn sign_stake(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_stake not implemented".to_string()))
    }

    fn sign_message(&self, _message: &[u8], _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_message not implemented".to_string()))
    }

    fn sign_account_action(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_account_action not implemented".to_string()))
    }

    fn sign_perpetual(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_perpetual not implemented".to_string()))
    }

    fn sign_withdrawal(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_withdrawal not implemented".to_string()))
    }

    fn sign_data(&self, _input: &TransactionLoadInput, _private_key: &[u8]) -> Result<String, SignerError> {
        Err(SignerError::UnsupportedOperation("sign_data not implemented".to_string()))
    }
}
