use primitives::{ChainSigner, SignerError, SignerInput, TransactionLoadInput};
use signer::Ed25519KeyPair;

use crate::address::PolkadotAddress;
use crate::transfer::NativeTransferTransaction;

#[derive(Default)]
pub struct PolkadotChainSigner;

impl ChainSigner for PolkadotChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        Self::sign_transaction(&input.input, private_key)
    }
}

impl PolkadotChainSigner {
    fn sign_transaction(input: &TransactionLoadInput, private_key: &[u8]) -> Result<String, SignerError> {
        let key_pair = Ed25519KeyPair::from_private_key(private_key)?;
        let sender = PolkadotAddress::parse(&input.sender_address)?;
        if sender.account_id() != &key_pair.public_key_bytes {
            return SignerError::invalid_input_err("Polkadot sender address does not match private key");
        }

        let transaction = NativeTransferTransaction::from_input(input, key_pair.public_key_bytes)?;
        let signing_payload = transaction.signing_payload();
        let signature = key_pair.sign(signing_payload.as_ref());
        Ok(transaction.encode_hex(&signature))
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, GasPriceType, TransactionFee, TransactionInputType, TransactionLoadMetadata};

    use super::*;

    const ADDRESS: &str = "15e6w4u9nH4Tb9HdJco2Zua4y5DpHb1hHXBKBGkUrLMTpuXo";

    fn metadata() -> TransactionLoadMetadata {
        TransactionLoadMetadata::Polkadot {
            sequence: 0,
            genesis_hash: "0x91b171bb158e2d3848fa23a9f1c25182fb8e20313b2c1eb49219da7a70ce90c3".to_string(),
            block_hash: "0x6e3ffeaa3be9d19bd110e5b6e7cbbc92cceed0d2ec557276c296bf7970ace2e5".to_string(),
            block_number: 24_666_537,
            spec_version: 1_003_004,
            transaction_version: 26,
            period: 64,
        }
    }

    fn input() -> SignerInput {
        let fee = TransactionFee::new_from_fee(10.into());
        SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Polkadot)),
                sender_address: ADDRESS.to_string(),
                destination_address: ADDRESS.to_string(),
                value: "10000".to_string(),
                gas_price: GasPriceType::regular(10),
                memo: None,
                is_max_value: false,
                metadata: metadata(),
            },
            fee,
        )
    }

    #[test]
    fn test_sign_transfer_matches_mobile_vector() {
        let private_key = hex::decode("f4c1daf4543e155b0e5e97351726d8891eae98014ed3f9a9ee1d842753c070ff").unwrap();

        assert_eq!(
            PolkadotChainSigner.sign_transfer(&input(), &private_key).unwrap(),
            "0x39028400cd3cfbbaa8f217c2a29ceae4b4063b597b629861916bad98f9826e03d1ab120\
            e00b2276e04c8adcd667512ec0440dd208f8ada56a4aec7572e4742ca2c0f8e5752d4d4f29d7\
            2a17c5d7e6bbfe2dfc9f081e567fdb9111be12ca04dec40cd2be0079502000000000a0000cd3\
            cfbbaa8f217c2a29ceae4b4063b597b629861916bad98f9826e03d1ab120e419c"
                .replace(char::is_whitespace, "")
        );
    }

    #[test]
    fn test_sign_transfer_rejects_sender_private_key_mismatch() {
        let private_key = hex::decode("f4c1daf4543e155b0e5e97351726d8891eae98014ed3f9a9ee1d842753c070ff").unwrap();
        let mut input = input();
        input.input.sender_address = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5".to_string();

        assert_eq!(
            PolkadotChainSigner.sign_transfer(&input, &private_key).unwrap_err().to_string(),
            "Invalid input: Polkadot sender address does not match private key"
        );
    }
}
