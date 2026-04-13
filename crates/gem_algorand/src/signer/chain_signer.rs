use crate::models::signing::AlgorandTransaction;
use crate::signer::signing::sign_transaction;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct AlgorandChainSigner;

impl ChainSigner for AlgorandChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&AlgorandTransaction::transfer(input)?, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&AlgorandTransaction::token_transfer(input)?, private_key)
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        sign_transaction(&AlgorandTransaction::account_action(input)?, private_key)
    }
}

#[cfg(test)]
mod tests {
    // Tests taken from https://github.com/trustwallet/wallet-core/blob/master/tests/chains/Algorand/TWAnySignerTests.cpp
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain, TransactionFee, TransactionLoadInput, TransactionLoadMetadata};

    const PRIVATE_KEY: &str = "5a6a3cfe5ff4cc44c19381d15a0d16de2a76ee5c9b9d83b232e38cb5a2c84b04";
    const SENDER: &str = "QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQU";
    const DESTINATION: &str = "GJIWJSX2EU5RC32LKTDDXWLA2YICBHKE35RV2ZPASXZYKWUWXFLKNFSS4U";

    #[test]
    fn test_sign_algorand_transactions() {
        let key = hex::decode(PRIVATE_KEY).unwrap();
        let token = Asset::new(AssetId::token(Chain::Algorand, "13379146"), "AlgoToken".into(), "ALGO".into(), 6, AssetType::TOKEN);
        let metadata = |sequence: u64| TransactionLoadMetadata::Algorand {
            sequence,
            block_hash: "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=".into(),
            chain_id: "testnet-v1.0".into(),
        };

        // Native transfer
        let input = SignerInput::new(
            TransactionLoadInput::mock_transfer(Asset::from_chain(Chain::Algorand), SENDER, DESTINATION, "1000000", 2340, None, metadata(15775683)),
            TransactionFee::new_from_fee(2340.into()),
        );
        let signed = AlgorandChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "82a3736967c440e87330ca542b7ee4f09ff31f8752e51c8a13fdf2b9d0c07a67a40ed3c4c981e4e23b1ea5f17cb5f34e5e66a937110f4ae5800baf09a12ea18dda25193c399d06a374786e89a3616d74ce000f4240a3666565cd0924a26676ce00f0b7c3a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bbaba3726376c420325164cafa253b116f4b54c63bd960d610209d44df635d65e095f3855a96b956a3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a3706179"
        );

        // Token transfer
        let input = SignerInput::new(
            TransactionLoadInput::mock_transfer(token.clone(), SENDER, DESTINATION, "1000000", 2340, None, metadata(15775683)),
            TransactionFee::new_from_fee(2340.into()),
        );
        let signed = AlgorandChainSigner.sign_token_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "82a3736967c440412720eff99a17280a437bdb8eeba7404b855d6433fffd5dde7f7966c1f9ae531a1af39e18b8a58b4a6c6acb709cca92f8a18c36d8328be9520c915311027005a374786e8aa461616d74ce000f4240a461726376c420325164cafa253b116f4b54c63bd960d610209d44df635d65e095f3855a96b956a3666565cd0924a26676ce00f0b7c3a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bbaba3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a56178666572a478616964ce00cc264a"
        );

        // Account action (asset opt-in)
        let input = SignerInput::new(
            TransactionLoadInput::mock_transfer(token, SENDER, "", "0", 2340, None, metadata(15775553)),
            TransactionFee::new_from_fee(2340.into()),
        );
        let signed = AlgorandChainSigner.sign_account_action(&input, &key).unwrap();
        assert_eq!(
            signed,
            "82a3736967c440f3a29d9a40271c00b542b38ab2ccb4967015ae6609368d4b8eb2f5e2b5348577cf9e0f62b0777ccb2d8d9b943b15c24c0cf1db312cb01a3c198d9d9c6c5bb00ba374786e89a461726376c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a3666565cd0924a26676ce00f0b741a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bb29a3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a56178666572a478616964ce00cc264a"
        );
    }

    #[test]
    fn test_sign_native_transfer_with_note() {
        let key = hex::decode("d5b43d706ef0cb641081d45a2ec213b5d8281f439f2425d1af54e2afdaabf55b").unwrap();
        let load = TransactionLoadInput::mock_transfer(
            Asset::from_chain(Chain::Algorand),
            "MG7QMDX4ALRIQ7P77SHNQUTIZDAJDQAT53PTCW6FA6KNAKUHSGW4FGK32Q",
            "CRLADAHJZEW2GFY2UPEHENLOGCUOU74WYSTUXQLVLJUJFHEUZOHYZNWYR4",
            "1000000000000",
            263000,
            Some("hello"),
            TransactionLoadMetadata::Algorand {
                sequence: 1937767,
                block_hash: "wGHE2Pwdvd7S12BL5FaOP20EGYesN73ktiC1qzkkit8=".into(),
                chain_id: "mainnet-v1.0".into(),
            },
        );
        let input = SignerInput::new(load, TransactionFee::new_from_fee(263000.into()));

        let signed = AlgorandChainSigner.sign_transfer(&input, &key).unwrap();
        assert_eq!(
            signed,
            "82a3736967c440baa00062adcdcb5875e4435cdc6885d26bfe5308ab17983c0fda790b7103051fcb111554e5badfc0ac7edf7e1223a434342a9eeed5cdb047690827325051560ba374786e8aa3616d74cf000000e8d4a51000a3666565ce00040358a26676ce001d9167a367656eac6d61696e6e65742d76312e30a26768c420c061c4d8fc1dbdded2d7604be4568e3f6d041987ac37bde4b620b5ab39248adfa26c76ce001d954fa46e6f7465c40568656c6c6fa3726376c42014560180e9c92da3171aa3c872356e30a8ea7f96c4a74bc1755a68929c94cb8fa3736e64c42061bf060efc02e2887dfffc8ed85268c8c091c013eedf315bc50794d02a8791ada474797065a3706179"
        );
    }
}
