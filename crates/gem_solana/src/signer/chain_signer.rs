use crate::decode_transaction;
use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{ChainSigner, SignerError, SignerInput, TransactionFee, TransferDataOutputType};
use solana_primitives::sign_message;

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_swap(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let tx_base64 = &swap_data.data.data;

        let unit_price = input.fee.unit_price_u64()?;
        let quote_gas_limit = swap_data
            .data
            .gas_limit
            .as_deref()
            .map(|value| value.parse::<u32>().map_err(|_| SignerError::invalid_input("invalid gas_limit")))
            .transpose()?;

        let signed = Self::sign_transaction(tx_base64, private_key, unit_price, quote_gas_limit, &input.fee)?;

        Ok(vec![signed])
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
        let signature = sign_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

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

impl SolanaChainSigner {
    fn sign_transaction(tx_base64: &str, private_key: &[u8], unit_price: u64, quote_gas_limit: Option<u32>, fee: &TransactionFee) -> Result<String, SignerError> {
        let mut tx = decode_transaction(tx_base64).map_err(SignerError::invalid_input)?;

        // Skip message modifications if co-signers present — changing the message would invalidate their signatures
        if tx.signatures().len() <= 1 {
            let gas_limit = Self::resolve_gas_limit(quote_gas_limit, tx.get_compute_unit_limit(), fee)?;
            if unit_price > 0 {
                tx.set_compute_unit_price(unit_price)
                    .map_err(|e| SignerError::invalid_input(format!("set compute unit price: {e}")))?;
            }
            if let Some(gas_limit) = gas_limit.filter(|gas_limit| *gas_limit > 0) {
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

        Ok(encode_base64(&bytes))
    }

    fn resolve_gas_limit(quote_gas_limit: Option<u32>, transaction_gas_limit: Option<u32>, fee: &TransactionFee) -> Result<Option<u32>, SignerError> {
        match quote_gas_limit.or(transaction_gas_limit) {
            Some(gas_limit) => Ok(Some(gas_limit)),
            None => {
                let gas_limit = fee.gas_limit.to_u64().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
                if gas_limit == 0 {
                    Ok(None)
                } else {
                    gas_limit.try_into().map(Some).map_err(|_| SignerError::invalid_input("invalid gas limit"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signer::testkit::*;
    use gem_encoding::decode_base64;
    use primitives::SwapProvider;
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransferDataOutputType};
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
        let signed_tx = VersionedTransaction::deserialize_with_version(&signed_bytes).unwrap();
        assert_eq!(signed_tx.signatures().len(), 1);
        assert_ne!(signed_tx.signatures()[0].as_bytes(), &[0u8; 64]);
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
    fn test_sign_swap_without_quote_gas_limit_uses_embedded_limit() {
        let signer = SolanaChainSigner;
        let original_limit = crate::decode_transaction(SINGLE_SIG_TX).unwrap().get_compute_unit_limit();
        let swap_data = SwapData::mock_with_provider_data(SwapProvider::Jupiter, SINGLE_SIG_TX, None);
        let input_type = TransactionInputType::Swap(Asset::mock_sol(), Asset::mock_spl_token(), swap_data);
        let input = TransactionLoadInput::mock_with_input_type(input_type);
        let fee = TransactionFee::new_gas_price_type(GasPriceType::solana(5_000u64, 0u64, 0u64), 5_000u64.into(), 1u64.into(), Default::default());
        let input = SignerInput::new(input, fee);

        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_tx = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_tx.get_compute_unit_limit(), original_limit);
        assert_ne!(signed_tx.signatures()[0].as_bytes(), &[0u8; 64]);
    }

    #[test]
    fn test_sign_swap_prefers_quote_gas_limit() {
        let signer = SolanaChainSigner;
        let gas_limit = crate::DEFAULT_SWAP_GAS_LIMIT.to_string();
        let swap_data = SwapData::mock_with_provider_data(SwapProvider::Jupiter, SINGLE_SIG_TX, Some(&gas_limit));
        let input_type = TransactionInputType::Swap(Asset::mock_sol(), Asset::mock_spl_token(), swap_data);
        let input = TransactionLoadInput::mock_with_input_type(input_type);
        let fee = TransactionFee::new_gas_price_type(GasPriceType::solana(5_000u64, 0u64, 0u64), 5_000u64.into(), 1u64.into(), Default::default());
        let input = SignerInput::new(input, fee);

        let result = signer.sign_swap(&input, &TEST_PRIVATE_KEY).unwrap();

        let signed_tx = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_tx.get_compute_unit_limit(), Some(crate::DEFAULT_SWAP_GAS_LIMIT));
    }
}
