use hex::decode;
use primitives::{ChainSigner, SignerError, TransactionInputType, TransactionLoadInput, stake_type::StakeType};

#[derive(Default)]
pub struct SuiChainSigner;

impl SuiChainSigner {
    fn sign_from_metadata(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_from_metadata(input, private_key)
    }

    fn ensure_supported_stake(&self, input: &TransactionLoadInput) -> Result<(), SignerError> {
        match &input.input_type {
            TransactionInputType::Stake(_, stake_type) => match stake_type {
                StakeType::Stake(_) | StakeType::Unstake(_) => Ok(()),
                StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) => Err(SignerError::UnsupportedOperation(
                    "Sui signer does not support this staking operation yet".to_string(),
                )),
                StakeType::Freeze(_) => Err(SignerError::InvalidInput("Sui does not support freeze operations".to_string())),
            },
            _ => Err(SignerError::InvalidInput("Expected stake transaction".to_string())),
        }
    }
}

impl ChainSigner for SuiChainSigner {
    fn sign_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }

    fn sign_token_transfer(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }

    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        self.sign_from_metadata(input, private_key).map(|signature| vec![signature])
    }

    fn sign_stake(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        self.ensure_supported_stake(input)?;
        self.sign_from_metadata(input, private_key).map(|signature| vec![signature])
    }

    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        self.sign_from_metadata(input, private_key)
    }
}

pub fn sign_from_metadata(input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
    let message_bytes = input.metadata.get_message_bytes().map_err(|err| SignerError::InvalidInput(err.to_string()))?;
    sign_message_bytes(&message_bytes, private_key)
}

pub fn sign_message_bytes(message: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let (prefix, digest_hex) = message
        .split_once('_')
        .ok_or_else(|| SignerError::InvalidInput("Invalid Sui digest payload".to_string()))?;

    let digest = decode(digest_hex.trim_start_matches("0x")).map_err(|_| SignerError::InvalidInput("Invalid digest hex for Sui transaction".to_string()))?;

    let signature = ::signer::Signer::sign_sui_digest(digest, private_key.to_vec()).map_err(|err| SignerError::InvalidInput(err.to_string()))?;

    Ok(format!("{prefix}_{signature}"))
}
