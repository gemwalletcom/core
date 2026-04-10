use crate::models::signing::StellarTransaction;
use crate::signer::signing::sign_transaction;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct StellarChainSigner;

impl ChainSigner for StellarChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::transfer(input)?, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::token_transfer(input)?, private_key)
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&StellarTransaction::account_action(input)?, private_key)
    }
}

#[cfg(test)]
mod tests {
    // Tests taken from https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Stellar/TWAnySignerTests.cpp
    use super::*;
    use crate::address::StellarAddress;
    use crate::models::signing::{Memo, Operation, StellarAssetData, StellarTransaction};
    use crate::signer::signing::sign_transaction;
    use gem_encoding::decode_base64;
    use primitives::{Address, Asset, AssetType, Chain, TransactionFee, TransactionLoadInput};
    use signer::Ed25519KeyPair;

    const PRIVATE_KEY: &str = "59a313f46ef1c23a9e4f71cea10fc0c56a2a6bb8a4b9ea3d5348823e5a478722";
    const SENDER: &str = "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI";
    const DESTINATION: &str = "GDCYBNRRPIHLHG7X7TKPUPAZ7WVUXCN3VO7WCCK64RIFV5XM5V5K4A52";

    #[test]
    fn test_sign_stellar_transactions() {
        let key = hex::decode(PRIVATE_KEY).unwrap();

        // Native transfer with memo
        let input = SignerInput::new(
            TransactionLoadInput::mock_stellar(Asset::from_chain(Chain::Stellar), SENDER, DESTINATION, "10000000", 1000, Some("Hello, world!"), 2, true),
            TransactionFee::new_from_fee(1000.into()),
        );
        let signed = StellarChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAEAAAANSGVsbG8sIHdvcmxkIQAAAAAAAAEAAAAAAAAAAQAAAADFgLYxeg6zm/f81Po8Gf2rS4m7q79hCV7kUFr27O16rgAAAAAAAAAAAJiWgAAAAAAAAAABGd9cogAAAEBQQldEkYJ6rMvOHilkwFCYyroGGUvrNeWVqr/sn3iFFqgz91XxgUT0ou7bMSPRgPROfBYDfQCFfFxbcDPrrCwB"
        );

        // Transfer to non-existent destination (creates account)
        let input = SignerInput::new(
            TransactionLoadInput::mock_stellar(Asset::from_chain(Chain::Stellar), SENDER, DESTINATION, "10000000", 1000, None, 2, false),
            TransactionFee::new_from_fee(1000.into()),
        );
        let signed = StellarChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAxYC2MXoOs5v3/NT6PBn9q0uJu6u/YQle5FBa9uzteq4AAAAAAJiWgAAAAAAAAAABGd9cogAAAEA6vrVXe4OUNPKKlGtzJiNzGi1p1yAd6pxoTEcoixXZbWponp6L5XOVweg5tTM36pZVQjQIxEjOgktinR96Wf8O"
        );

        // Token transfer
        let mobi = Asset::mock_with_params(
            Chain::Stellar,
            Some("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH::MOBI".into()),
            "MOBI".into(),
            "MOBI".into(),
            7,
            AssetType::TOKEN,
        );
        let input = SignerInput::new(
            TransactionLoadInput::mock_stellar(
                mobi,
                "GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL",
                "GA3ISGYIE2ZTH3UAKEKBVHBPKUSL3LT4UQ6C5CUGP2IM5F467O267KI7",
                "12000000",
                1000,
                None,
                144098454883270661,
                true,
            ),
            TransactionFee::new_from_fee(1000.into()),
        );
        let signed = StellarChainSigner
            .sign_token_transfer(&input, &hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap())
            .unwrap();
        assert_eq!(
            signed,
            "AAAAAMpFJQVVMv16RJUPlzQUTlgZOHVurhw3igGacP1305F1AAAD6AH/8MgAAAAFAAAAAAAAAAAAAAABAAAAAAAAAAEAAAAANokbCCazM+6AURQanC9VJL2ufKQ8LoqGfpDOl577te8AAAABTU9CSQAAAAA8cTArnmXa4wEQJxDHOw5SwBaDVjBfAP5lRMNZkRtlZAAAAAAAtxsAAAAAAAAAAAF305F1AAAAQEuWZZvKZuF6SMuSGIyfLqx5sn5O55+Kd489uP4g9jZH4UE7zZ4ME0+74I0BU8YDsYOmmxcfp/vdwTd+n3oGCQw="
        );
    }

    #[test]
    fn test_sign_change_trust_with_time_bounds() {
        let transaction = StellarTransaction {
            account: StellarAddress::from_str("GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL").unwrap(),
            fee: 10000,
            sequence: 144098454883270659,
            memo: Memo::None,
            time_bounds: None,
            operation: Operation::ChangeTrust {
                asset: StellarAssetData::new("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH", "MOBI").unwrap(),
                valid_before: Some(1613336576),
            },
        };

        let signed = sign_transaction(&transaction, &hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap()).unwrap();
        assert_eq!(
            signed,
            "AAAAAMpFJQVVMv16RJUPlzQUTlgZOHVurhw3igGacP1305F1AAAnEAH/8MgAAAADAAAAAQAAAAAAAAAAAAAAAGApkAAAAAAAAAAAAQAAAAAAAAAGAAAAAU1PQkkAAAAAPHEwK55l2uMBECcQxzsOUsAWg1YwXwD+ZUTDWZEbZWR//////////wAAAAAAAAABd9ORdQAAAEAnfyXyaNQX5Bq3AEQVBIaYd+cLib+y2sNY7DF/NYVSE51dZ6swGGElz094ObsPefmVmeRrkGsSc/fF5pmth+wJ"
        );
    }

    #[test]
    fn test_stellar_signing_validation_and_hint() {
        let transfer_key = hex::decode("3c0635f8638605aed6e461cf3fa2d508dd895df1a1655ff92c79bfbeaf88d4b9").unwrap();

        let signed = StellarChainSigner
            .sign_transfer(
                &SignerInput::new(
                    TransactionLoadInput::mock_stellar(Asset::from_chain(Chain::Stellar), SENDER, DESTINATION, "10000000", 1000, None, 2, true),
                    TransactionFee::new_from_fee(1000.into()),
                ),
                &transfer_key,
            )
            .unwrap();
        let envelope = decode_base64(&signed).unwrap();
        let signer_key = Ed25519KeyPair::from_private_key(&transfer_key).unwrap();
        let sender = StellarAddress::from_str(SENDER).unwrap();
        let hint_offset = envelope.len() - 72;

        assert_eq!(&envelope[hint_offset..hint_offset + 4], &signer_key.public_key_bytes[28..32]);
        assert_ne!(&envelope[hint_offset..hint_offset + 4], &sender.as_bytes()[28..32]);
    }
}
