use gem_hash::blake2::blake2b_224;
use primitives::{ChainSigner, SignerError, SignerInput};

use crate::{
    address::ShelleyAddress,
    planner::{plan_transfer, transaction_from_plan},
};

use super::extended_key::CardanoExtendedKeyPair;

pub struct CardanoChainSigner;

impl ChainSigner for CardanoChainSigner {
    fn sign_transfer(&self, input: &SignerInput, private_key: &[u8]) -> Result<String, SignerError> {
        let plan = plan_transfer(&input.input)?;
        let key_pair = CardanoExtendedKeyPair::from_private_key(private_key)?;
        let public_key = key_pair.public_key();
        let sender = ShelleyAddress::parse(&input.sender_address)?;
        let payment_hash = blake2b_224(&public_key);
        if sender.payment_hash() != payment_hash.as_slice() {
            return SignerError::invalid_input_err("Cardano private key does not match sender address");
        }
        let transaction = transaction_from_plan(&input.input, &plan)?;
        let transaction_id = transaction.transaction_id();
        let signature = key_pair.sign(&transaction_id);

        Ok(hex::encode(transaction.signed_bytes(&public_key, &signature)))
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, GasPriceType, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput, TransactionLoadMetadata, UTXO};

    use super::*;

    const PRIVATE_KEY_TEST_1: &str = "089b68e458861be0c44bf9f7967f05cc91e51ede86dc679448a3566990b7785bd48c330875b1e0d03caaed0e67cecc42075dce1c7a13b1c49240508848ac82f603391c68824881ae3fc23a56a1a75ada3b96382db502e37564e84a5413cfaf1290dbd508e5ec71afaea98da2df1533c22ef02a26bb87b31907d0b2738fb7785b38d53aa68fc01230784c9209b2b2a2faf28491b3b1f1d221e63e704bbd0403c4154425dfbb01a2c5c042da411703603f89af89e57faae2946e2a5c18b1c5ca0e";
    const WALLET_CORE_OWN_ADDRESS_1: &str = "addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23";
    const TO_ADDRESS: &str = "addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5";
    const TEST_BLOCK_NUMBER: u64 = 189_992_800;

    fn signer_input(sender_address: &str) -> SignerInput {
        SignerInput::new(
            TransactionLoadInput {
                input_type: TransactionInputType::Transfer(Asset::from_chain(Chain::Cardano)),
                sender_address: sender_address.to_string(),
                destination_address: TO_ADDRESS.to_string(),
                value: "7000000".to_string(),
                gas_price: GasPriceType::regular(0u64),
                memo: None,
                is_max_value: false,
                metadata: TransactionLoadMetadata::Cardano {
                    block_number: TEST_BLOCK_NUMBER,
                    utxos: vec![
                        UTXO {
                            transaction_id: "f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767".to_string(),
                            vout: 1,
                            value: "1500000".to_string(),
                            address: sender_address.to_string(),
                        },
                        UTXO {
                            transaction_id: "554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af0".to_string(),
                            vout: 0,
                            value: "6500000".to_string(),
                            address: sender_address.to_string(),
                        },
                    ],
                },
            },
            TransactionFee::default(),
        )
    }

    #[test]
    fn test_sign_transfer_vector() {
        let input = signer_input(WALLET_CORE_OWN_ADDRESS_1);
        let private_key = hex::decode(PRIVATE_KEY_TEST_1).unwrap();
        let plan = plan_transfer(&input.input).unwrap();
        let key_pair = CardanoExtendedKeyPair::from_private_key(&private_key).unwrap();

        let mut transaction = transaction_from_plan(&input.input, &plan).unwrap();
        transaction.expiration_block_number = 53_333_333;
        let transaction_id = transaction.transaction_id();
        let signature = key_pair.sign(&transaction_id);

        assert_eq!(
            hex::encode(transaction.signed_bytes(&key_pair.public_key(), &signature)),
            "84a40082825820554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af000825820f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e76701018182583901558dd902616f5cd01edcc62870cb4748c45403f1228218bee5b628b526f0ca9e7a2c04d548fbd6ce86f358be139fe680652536437d1d6fd51a006acfc0021a000f4240031a032dcd55a100818258206d8a0b425bd2ec9692af39b1c0cf0e51caa07a603550e22f54091e872c7df29058407519e5e7391f8a47f58c8ded1ce532dc80910ef25b108b1092cb58e86a318964956d00af763087fddabf631d00508d1e9c206eaf762176f538042f5c52f6d902f5f6"
        );
        assert_eq!(hex::encode(transaction_id), "92859ce37002afc9185c5a918e6596b90258dd3b59ea686ec625bf1b15a5c101");
    }

    #[test]
    fn test_sign_transfer_rejects_invalid_key_length() {
        let input = signer_input(WALLET_CORE_OWN_ADDRESS_1);
        assert!(CardanoChainSigner.sign_transfer(&input, &[0u8; 32]).is_err());
    }

    #[test]
    fn test_sign_transfer_rejects_sender_key_mismatch() {
        let input = signer_input(TO_ADDRESS);
        let private_key = hex::decode(PRIVATE_KEY_TEST_1).unwrap();
        assert_eq!(
            CardanoChainSigner.sign_transfer(&input, &private_key).err().unwrap().to_string(),
            "Invalid input: Cardano private key does not match sender address"
        );
    }
}
