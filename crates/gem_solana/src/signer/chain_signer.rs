use base64::{Engine, engine::general_purpose::STANDARD};
use num_traits::ToPrimitive;
use primitives::{ChainSigner, SignerError, TransactionLoadInput, TransferDataOutputType};
use solana_primitives::{VersionedTransaction, sign_message};

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let tx_base64 = &swap_data.data.data;

        let unit_price: u64 = input.gas_price.unit_price().to_u64().unwrap_or(0);
        let gas_limit = swap_data.data.gas_limit_as_u32().map_err(SignerError::invalid_input)?;

        let signed = Self::sign_transaction(tx_base64, private_key, unit_price, gas_limit)?;

        Ok(vec![signed])
    }

    fn sign_data(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let extra = input.input_type.get_generic_data().map_err(SignerError::invalid_input)?;
        let tx_bytes = STANDARD
            .decode(extra.data_as_str().map_err(SignerError::invalid_input)?)
            .map_err(|e| SignerError::invalid_input(format!("base64 decode: {e}")))?;

        let mut transaction = VersionedTransaction::deserialize_with_version(&tx_bytes).map_err(|e| SignerError::invalid_input(format!("parse transaction: {e}")))?;

        let signatures = transaction.signatures();
        if signatures.is_empty() || signatures[0].as_bytes() != &[0u8; 64] {
            return Err(SignerError::invalid_input("user signature should be first"));
        }

        let message_bytes = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
        let signature = sign_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

        match extra.output_type {
            TransferDataOutputType::Signature => Ok(bs58::encode(signature.as_bytes()).into_string()),
            TransferDataOutputType::EncodedTransaction => {
                transaction.signatures_mut()[0] = signature;
                let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
                Ok(STANDARD.encode(&bytes))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::testkit::*;
    use primitives::{Chain, TransactionLoadInput, TransferDataOutputType};

    #[test]
    fn test_deserialize_single_signature_transaction() {
        let bytes = STANDARD.decode(SINGLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 1);

        let message_bytes = transaction.serialize_message().unwrap();
        let message_hex: String = message_bytes.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(message_hex, EXPECTED_MESSAGE_HEX);
    }

    #[test]
    fn test_deserialize_double_signature_transaction() {
        let bytes = STANDARD.decode(DOUBLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&bytes).unwrap();

        assert_eq!(transaction.signatures().len(), 2);
    }

    #[test]
    fn test_sign_data_encoded_transaction() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::EncodedTransaction);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_bytes = STANDARD.decode(&result).unwrap();
        let signed_tx = VersionedTransaction::deserialize_with_version(&signed_bytes).unwrap();
        assert_eq!(signed_tx.signatures().len(), 1);
        assert_ne!(signed_tx.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_data_signature_output() {
        let signer = SolanaChainSigner;
        let input = TransactionLoadInput::mock_sign_data(Chain::Solana, SINGLE_SIG_TX, TransferDataOutputType::Signature);

        let result = signer.sign_data(&input, &TEST_PRIVATE_KEY).unwrap();

        let sig_bytes = bs58::decode(&result).into_vec().unwrap();
        assert_eq!(sig_bytes.len(), 64);
    }
}

impl SolanaChainSigner {
    fn sign_transaction(tx_base64: &str, private_key: &[u8], unit_price: u64, gas_limit: u32) -> Result<String, SignerError> {
        let data = STANDARD.decode(tx_base64).map_err(|e| SignerError::invalid_input(format!("base64 decode: {e}")))?;

        let mut tx = VersionedTransaction::deserialize_with_version(&data).map_err(|e| SignerError::invalid_input(format!("parse transaction: {e}")))?;

        // Skip message modifications if co-signers present — changing the message would invalidate their signatures
        if tx.signatures().len() <= 1 {
            if unit_price > 0 {
                tx.set_compute_unit_price(unit_price)
                    .map_err(|e| SignerError::invalid_input(format!("set compute unit price: {e}")))?;
            }
            if gas_limit > 0 {
                tx.set_compute_unit_limit(gas_limit)
                    .map_err(|e| SignerError::invalid_input(format!("set compute unit limit: {e}")))?;
            }
        }

        let message_bytes = tx.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;

        let sig = sign_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

        let sigs = tx.signatures_mut();
        if sigs.is_empty() {
            sigs.push(sig);
        } else {
            sigs[0] = sig;
        }

        let bytes = tx.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;

        Ok(STANDARD.encode(&bytes))
    }
}
