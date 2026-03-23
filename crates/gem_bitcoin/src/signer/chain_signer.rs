use primitives::{ChainSigner, SignerError, SwapProvider, TransactionLoadInput};

use super::psbt::sign_psbt;

#[derive(Default)]
pub struct BitcoinChainSigner;

impl ChainSigner for BitcoinChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let provider = &swap_data.quote.provider_data.provider;

        match provider {
            SwapProvider::Relay => {
                let psbt_hex = &swap_data.data.data;
                let signed = sign_psbt(psbt_hex, private_key)?;
                Ok(vec![signed])
            }
            SwapProvider::Thorchain | SwapProvider::Chainflip => Err(SignerError::signing_error("bitcoin transfer swaps not yet implemented in Rust")),
            other => Err(SignerError::signing_error(format!("unsupported swap provider for Bitcoin: {:?}", other))),
        }
    }
}
