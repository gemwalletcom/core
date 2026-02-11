use base64::{Engine, engine::general_purpose::STANDARD};
use num_traits::ToPrimitive;
use primitives::{ChainSigner, SignerError, TransactionLoadInput};
use solana_primitives::{VersionedTransaction, sign_message};

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let tx_base64 = &swap_data.data.data;

        let unit_price: u64 = input.gas_price.unit_price().to_u64().unwrap_or(0);
        let gas_limit = input.gas_limit;

        let signed = Self::sign_transaction(tx_base64, private_key, unit_price, gas_limit)?;

        Ok(vec![signed])
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
