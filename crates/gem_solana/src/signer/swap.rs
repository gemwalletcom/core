use crate::decode_transaction;
use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};
use solana_primitives::sign_message as sign_solana_message;

pub(crate) fn sign(input: &SignerInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
    let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
    let transaction_base64 = &swap_data.data.data;

    let unit_price = input.fee.unit_price_u64()?;
    let quote_gas_limit = swap_data
        .data
        .gas_limit
        .as_ref()
        .map(|_| swap_data.data.gas_limit_as_u32())
        .transpose()
        .map_err(SignerError::invalid_input)?;

    Ok(vec![sign_transaction(transaction_base64, private_key, unit_price, quote_gas_limit, &input.fee)?])
}

fn sign_transaction(transaction_base64: &str, private_key: &[u8], unit_price: u64, quote_gas_limit: Option<u32>, fee: &TransactionFee) -> Result<String, SignerError> {
    let mut transaction = decode_transaction(transaction_base64).map_err(SignerError::invalid_input)?;

    if transaction.signatures().len() <= 1 {
        let gas_limit = match quote_gas_limit.or(transaction.get_compute_unit_limit()) {
            Some(gas_limit) => Some(gas_limit),
            None => {
                let gas_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
                (gas_limit > 0).then_some(gas_limit)
            }
        };
        if unit_price > 0 {
            transaction
                .set_compute_unit_price(unit_price)
                .map_err(|e| SignerError::invalid_input(format!("set compute unit price: {e}")))?;
        }
        if let Some(gas_limit) = gas_limit.filter(|gas_limit| *gas_limit > 0) {
            transaction
                .set_compute_unit_limit(gas_limit)
                .map_err(|e| SignerError::invalid_input(format!("set compute unit limit: {e}")))?;
        }
    }

    let message_bytes = transaction.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;
    let sig = sign_solana_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

    let sigs = transaction.signatures_mut();
    if sigs.is_empty() {
        sigs.push(sig);
    } else {
        sigs[0] = sig;
    }

    let bytes = transaction.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
    Ok(encode_base64(&bytes))
}

#[cfg(test)]
mod tests {
    use crate::signer::{SolanaChainSigner, testkit::SINGLE_SIG_TX};
    use primitives::swap::SwapData;
    use primitives::testkit::signer_mock::TEST_PRIVATE_KEY;
    use primitives::{Asset, ChainSigner, GasPriceType, SignerInput, SwapProvider, TransactionFee, TransactionInputType, TransactionLoadInput};

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

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), original_limit);
        assert_ne!(signed_transaction.signatures()[0].as_bytes(), &[0u8; 64]);
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

        let signed_transaction = crate::decode_transaction(&result[0]).unwrap();
        assert_eq!(signed_transaction.get_compute_unit_limit(), Some(crate::DEFAULT_SWAP_GAS_LIMIT));
    }
}
