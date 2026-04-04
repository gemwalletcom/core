use crate::signer::models::NearTransfer;
use crate::signer::signature;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct NearChainSigner;

impl ChainSigner for NearChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = NearTransfer::from_input(input)?;
        signature::sign_transfer(&transaction, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};

    const PRIVATE_KEY: &str = "3hoMW1HvnRLSFCLZnvPzWeoGwtdHzke34B2cTHM8rhcbG3TbuLKtShTv3DvyejnXKXKBiV7YPkLeqUHN1ghnqpFv";

    #[test]
    fn sign_transfer() {
        let private_key = bs58::decode(PRIVATE_KEY).into_vec().unwrap();

        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Near)),
                sender_address: "test.near".into(),
                destination_address: "whatever.near".into(),
                value: "1".into(),
                gas_price: GasPriceType::regular(0),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Near {
                    sequence: 1,
                    block_hash: "244ZQ9cgj3CQ6bWBdytfrJMuMQ1jdXLFGnr4HhvtCTnM".into(),
                },
            },
            TransactionFee::new_from_fee(0.into()),
        );

        let signed = NearChainSigner.sign_transfer(&input, &private_key[..32]).unwrap();

        assert_eq!(
            signed,
            "CQAAAHRlc3QubmVhcgCRez0mjUtY9/7BsVC9aNab4+5dTMOYVeNBU4Rlu3eGDQEAAAAAAAAADQAAAHdoYXRldmVyLm5lYXIPpHP9JpAd8pa+atxMxN800EDvokNSJLaYaRDmMML+9gEAAAADAQAAAAAAAAAAAAAAAAAAAACWmoMzIYbul1Xkg5MlUlgG4Ymj0tK7S0dg6URD6X4cTyLe7vAFmo6XExAO2m4ZFE2n6KDvflObIHCLodjQIb0B"
        );
    }
}
