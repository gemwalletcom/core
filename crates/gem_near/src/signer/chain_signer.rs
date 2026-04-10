use crate::signer::models::NearTransfer;
use crate::signer::signing;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct NearChainSigner;

impl ChainSigner for NearChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = NearTransfer::from_input(input)?;
        signing::sign_transfer(&transaction, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{TransactionFee, TransactionLoadInput};

    // Tests taken from https://github.com/trustwallet/wallet-core/blob/master/tests/chains/NEAR/SignerTests.cpp
    #[test]
    fn test_sign_near_transfer() {
        let private_key = bs58::decode("3hoMW1HvnRLSFCLZnvPzWeoGwtdHzke34B2cTHM8rhcbG3TbuLKtShTv3DvyejnXKXKBiV7YPkLeqUHN1ghnqpFv")
            .into_vec()
            .unwrap();

        let input = SignerInput::new(
            TransactionLoadInput::mock_near("test.near", "whatever.near", "1", 1, "244ZQ9cgj3CQ6bWBdytfrJMuMQ1jdXLFGnr4HhvtCTnM"),
            TransactionFee::new_from_fee(0.into()),
        );

        let signed = NearChainSigner.sign_transfer(&input, &private_key[..32]).unwrap();

        assert_eq!(
            signed,
            "CQAAAHRlc3QubmVhcgCRez0mjUtY9/7BsVC9aNab4+5dTMOYVeNBU4Rlu3eGDQEAAAAAAAAADQAAAHdoYXRldmVyLm5lYXIPpHP9JpAd8pa+atxMxN800EDvokNSJLaYaRDmMML+9gEAAAADAQAAAAAAAAAAAAAAAAAAAACWmoMzIYbul1Xkg5MlUlgG4Ymj0tK7S0dg6URD6X4cTyLe7vAFmo6XExAO2m4ZFE2n6KDvflObIHCLodjQIb0B"
        );
    }
}
