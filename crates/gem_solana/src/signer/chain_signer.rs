use super::{instructions, swap, transaction};
use crate::decode_transaction;
use gem_encoding::encode_base64;
use primitives::{ChainSigner, SignerError, SignerInput, TransferDataOutputType};
use solana_primitives::{Pubkey, sign_message as sign_solana_message};

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::native_transfer(input, sender)?)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::token_transfer(input, sender)?)
    }

    fn sign_nft_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        transaction::sign_single_signer_instructions(input, private_key, sender, instructions::nft_transfer(input, sender)?)
    }

    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        swap::sign(input, private_key)
    }

    fn sign_stake(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let sender = Pubkey::from_base58(&input.sender_address).map_err(SignerError::from_display)?;
        Ok(vec![transaction::sign_single_signer_instructions(
            input,
            private_key,
            sender,
            instructions::stake(input, sender)?,
        )?])
    }

    fn sign_message(&self, message: &[u8], private_key: &[u8]) -> Result<String, SignerError> {
        let signature = sign_solana_message(private_key, message).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;
        Ok(bs58::encode(signature.as_bytes()).into_string())
    }

    fn sign_data(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        let data = extra.data_as_str().map_err(SignerError::invalid_input)?;
        let mut transaction = decode_transaction(data).map_err(SignerError::invalid_input)?;

        let signatures = transaction.signatures();
        if signatures.is_empty() || signatures[0].as_bytes() != &[0u8; 64] {
            return Err(SignerError::invalid_input("user signature should be first"));
        }

        let message_bytes = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
        let signature = sign_solana_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

        match extra.output_type {
            TransferDataOutputType::Signature => Ok(bs58::encode(signature.as_bytes()).into_string()),
            TransferDataOutputType::EncodedTransaction => {
                transaction.signatures_mut()[0] = signature;
                let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
                Ok(encode_base64(&bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::testkit::{DOUBLE_SIG_TX, EXPECTED_MESSAGE_HEX, SINGLE_SIG_TX};
    use gem_encoding::decode_base64;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Chain, ChainSigner, SignerInput, TransactionLoadInput, TransferDataOutputType};
    use solana_primitives::VersionedTransaction;

    #[test]
    fn test_deserialize_single_signature_transaction() {
        let bytes = decode_base64(SINGLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 1);

        let message_bytes = transaction.serialize_message().unwrap();
        let message_hex: String = message_bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(message_hex, EXPECTED_MESSAGE_HEX);
    }

    #[test]
    fn test_deserialize_double_signature_transaction() {
        let bytes = decode_base64(DOUBLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 2);
    }

    #[test]
    fn test_sign_data_encoded_transaction() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::EncodedTransaction);
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_bytes = decode_base64(&result).unwrap();
        let signed_transaction = VersionedTransaction::deserialize_with_version(&signed_bytes).unwrap();
        assert_eq!(signed_transaction.signatures().len(), 1);
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_data_signature_output() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::Signature);
        let fee = input.default_fee();
        let input = SignerInput::new(input, fee);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let sig_bytes = bs58::decode(&result).into_vec().unwrap();
        assert_eq!(sig_bytes.len(), 64);
    }

    #[test]
    fn test_sign_message() {
        let signer = SolanaChainSigner;

        let result = signer.sign_message(b"hello", &TEST_PRIVATE_KEY).unwrap();

        assert_eq!(bs58::decode(result).into_vec().unwrap().len(), 64);
    }
}
