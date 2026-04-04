use crate::models::signing::StellarTransaction;
use crate::signer::signature::sign_transaction;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct StellarChainSigner;

impl ChainSigner for StellarChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = StellarTransaction::transfer(input)?;
        sign_transaction(&transaction, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = StellarTransaction::token_transfer(input)?;
        sign_transaction(&transaction, private_key)
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = StellarTransaction::account_action(input)?;
        sign_transaction(&transaction, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::parse_address;
    use crate::models::signing::{Memo, Operation, StellarAssetData, StellarTransaction};
    use primitives::{Asset, AssetType, Chain, FeeOption, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};
    use std::collections::HashMap;

    const PRIVATE_KEY: &str = "59a313f46ef1c23a9e4f71cea10fc0c56a2a6bb8a4b9ea3d5348823e5a478722";

    #[test]
    fn sign_transfer_with_memo() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
                sender_address: "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI".into(),
                destination_address: "GDCYBNRRPIHLHG7X7TKPUPAZ7WVUXCN3VO7WCCK64RIFV5XM5V5K4A52".into(),
                value: "10000000".into(),
                gas_price: GasPriceType::regular(1000),
                memo: Some("Hello, world!".into()),
                is_max_value: false,
                metadata: TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: true,
                },
            },
            TransactionFee::new_from_fee(1000.into()),
        );

        let signed = StellarChainSigner.sign_transfer(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAEAAAANSGVsbG8sIHdvcmxkIQAAAAAAAAEAAAAAAAAAAQAAAADFgLYxeg6zm/f81Po8Gf2rS4m7q79hCV7kUFr27O16rgAAAAAAAAAAAJiWgAAAAAAAAAABGd9cogAAAEBQQldEkYJ6rMvOHilkwFCYyroGGUvrNeWVqr/sn3iFFqgz91XxgUT0ou7bMSPRgPROfBYDfQCFfFxbcDPrrCwB"
        );
    }

    #[test]
    fn sign_token_transfer() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::mock_with_params(
                    Chain::Stellar,
                    Some("GA6HCMBLTZS5VYYBCATRBRZ3BZJMAFUDKYYF6AH6MVCMGWMRDNSWJPIH::MOBI".into()),
                    "MOBI".into(),
                    "MOBI".into(),
                    7,
                    AssetType::TOKEN,
                )),
                sender_address: "GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL".into(),
                destination_address: "GA3ISGYIE2ZTH3UAKEKBVHBPKUSL3LT4UQ6C5CUGP2IM5F467O267KI7".into(),
                value: "12000000".into(),
                gas_price: GasPriceType::regular(1000),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Stellar {
                    sequence: 144098454883270661,
                    is_destination_address_exist: true,
                },
            },
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
    fn create_account_encoding() {
        let transaction = StellarTransaction {
            account: parse_address("GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI").unwrap(),
            fee: 1000,
            sequence: 2,
            memo: Memo::Id(1234567890),
            time_bounds: None,
            operation: Operation::CreateAccount {
                destination: parse_address("GDCYBNRRPIHLHG7X7TKPUPAZ7WVUXCN3VO7WCCK64RIFV5XM5V5K4A52").unwrap(),
                amount: 10_000_000,
            },
        };

        let signed = sign_transaction(&transaction, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_eq!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAIAAAAASZYC0gAAAAEAAAAAAAAAAAAAAADFgLYxeg6zm/f81Po8Gf2rS4m7q79hCV7kUFr27O16rgAAAAAAmJaAAAAAAAAAAAEZ31yiAAAAQNgqNDqbe0X60gyH+1xf2Tv2RndFiJmyfbrvVjsTfjZAVRrS2zE9hHlqPQKpZkGKEFka7+1ElOS+/m/1JDnauQg="
        );
    }

    #[test]
    fn change_trust_encoding() {
        let transaction = StellarTransaction {
            account: parse_address("GDFEKJIFKUZP26SESUHZONAUJZMBSODVN2XBYN4KAGNHB7LX2OIXLPUL").unwrap(),
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
    fn sign_transfer_uses_create_account_when_fee_option_is_present() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Stellar)),
                sender_address: "GAE2SZV4VLGBAPRYRFV2VY7YYLYGYIP5I7OU7BSP6DJT7GAZ35OKFDYI".into(),
                destination_address: "GDCYBNRRPIHLHG7X7TKPUPAZ7WVUXCN3VO7WCCK64RIFV5XM5V5K4A52".into(),
                value: "10000000".into(),
                gas_price: GasPriceType::regular(1000),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Stellar {
                    sequence: 2,
                    is_destination_address_exist: false,
                },
            },
            TransactionFee::new_gas_price_type(
                GasPriceType::regular(1000),
                1000.into(),
                0.into(),
                HashMap::from([(FeeOption::TokenAccountCreation, 0.into())]),
            ),
        );

        let signed = StellarChainSigner.sign_transfer(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_ne!(
            signed,
            "AAAAAAmpZryqzBA+OIlrquP4wvBsIf1H3U+GT/DTP5gZ31yiAAAD6AAAAAAAAAACAAAAAAAAAAAAAAABAAAAAAAAAAEAAAAAxYC2MXoOs5v3/NT6PBn9q0uJu6u/YQle5FBa9uzteq4AAAAAAAAAAACYloAAAAAAAAAAARnfXKIAAABAocQZwTnVvGMQlpdGacWvgenxN5ku8YB8yhEGrDfEV48yDqcj6QaePAitDj/N2gxfYD9Q2pJ+ZpkQMsZZG4ACAg=="
        );
    }
}
