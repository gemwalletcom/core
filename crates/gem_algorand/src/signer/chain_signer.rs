use crate::models::signing::AlgorandTransaction;
use crate::signer::signature::sign_transaction;
use primitives::{ChainSigner, SignerError, SignerInput};

#[derive(Default)]
pub struct AlgorandChainSigner;

impl ChainSigner for AlgorandChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = AlgorandTransaction::transfer(input)?;
        sign_transaction(&transaction, private_key)
    }

    fn sign_token_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = AlgorandTransaction::token_transfer(input)?;
        sign_transaction(&transaction, private_key)
    }

    fn sign_account_action(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let transaction = AlgorandTransaction::account_action(input)?;
        sign_transaction(&transaction, private_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Asset, AssetId, AssetType, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata};

    const PRIVATE_KEY: &str = "5a6a3cfe5ff4cc44c19381d15a0d16de2a76ee5c9b9d83b232e38cb5a2c84b04";
    const SENDER_ADDRESS: &str = "QKDS2YGDHDFZFAAGA4HAF3AJIKW5ZN46P66QDR3ELCXKKJUJTPJSXVHNQU";
    const DESTINATION_ADDRESS: &str = "GJIWJSX2EU5RC32LKTDDXWLA2YICBHKE35RV2ZPASXZYKWUWXFLKNFSS4U";

    #[test]
    fn sign_transfer() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Algorand)),
                sender_address: SENDER_ADDRESS.into(),
                destination_address: DESTINATION_ADDRESS.into(),
                value: "1000000".into(),
                gas_price: GasPriceType::regular(2340),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Algorand {
                    sequence: 15775683,
                    block_hash: "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=".into(),
                    chain_id: "testnet-v1.0".into(),
                },
            },
            TransactionFee::new_from_fee(2340.into()),
        );

        let signed = AlgorandChainSigner.sign_transfer(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_eq!(
            signed,
            "82a3736967c440e87330ca542b7ee4f09ff31f8752e51c8a13fdf2b9d0c07a67a40ed3c4c981e4e23b1ea5f17cb5f34e5e66a937110f4ae5800baf09a12ea18dda25193c399d06a374786e89a3616d74ce000f4240a3666565cd0924a26676ce00f0b7c3a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bbaba3726376c420325164cafa253b116f4b54c63bd960d610209d44df635d65e095f3855a96b956a3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a3706179"
        );
    }

    #[test]
    fn sign_token_transfer() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::new(
                    AssetId::token(Chain::Algorand, "13379146"),
                    "AlgoToken".into(),
                    "ALGO".into(),
                    6,
                    AssetType::TOKEN,
                )),
                sender_address: SENDER_ADDRESS.into(),
                destination_address: DESTINATION_ADDRESS.into(),
                value: "1000000".into(),
                gas_price: GasPriceType::regular(2340),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Algorand {
                    sequence: 15775683,
                    block_hash: "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=".into(),
                    chain_id: "testnet-v1.0".into(),
                },
            },
            TransactionFee::new_from_fee(2340.into()),
        );

        let signed = AlgorandChainSigner.sign_token_transfer(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_eq!(
            signed,
            "82a3736967c440412720eff99a17280a437bdb8eeba7404b855d6433fffd5dde7f7966c1f9ae531a1af39e18b8a58b4a6c6acb709cca92f8a18c36d8328be9520c915311027005a374786e8aa461616d74ce000f4240a461726376c420325164cafa253b116f4b54c63bd960d610209d44df635d65e095f3855a96b956a3666565cd0924a26676ce00f0b7c3a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bbaba3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a56178666572a478616964ce00cc264a"
        );
    }

    #[test]
    fn sign_account_action() {
        let input = SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::new(
                    AssetId::token(Chain::Algorand, "13379146"),
                    "AlgoToken".into(),
                    "ALGO".into(),
                    6,
                    AssetType::TOKEN,
                )),
                sender_address: SENDER_ADDRESS.into(),
                destination_address: String::new(),
                value: "0".into(),
                gas_price: GasPriceType::regular(2340),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Algorand {
                    sequence: 15775553,
                    block_hash: "SGO1GKSzyE7IEPItTxCByw9x8FmnrCDexi9/cOUJOiI=".into(),
                    chain_id: "testnet-v1.0".into(),
                },
            },
            TransactionFee::new_from_fee(2340.into()),
        );

        let signed = AlgorandChainSigner.sign_account_action(&input, &hex::decode(PRIVATE_KEY).unwrap()).unwrap();

        assert_eq!(
            signed,
            "82a3736967c440f3a29d9a40271c00b542b38ab2ccb4967015ae6609368d4b8eb2f5e2b5348577cf9e0f62b0777ccb2d8d9b943b15c24c0cf1db312cb01a3c198d9d9c6c5bb00ba374786e89a461726376c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a3666565cd0924a26676ce00f0b741a367656eac746573746e65742d76312e30a26768c4204863b518a4b3c84ec810f22d4f1081cb0f71f059a7ac20dec62f7f70e5093a22a26c76ce00f0bb29a3736e64c42082872d60c338cb928006070e02ec0942addcb79e7fbd01c76458aea526899bd3a474797065a56178666572a478616964ce00cc264a"
        );
    }
}
