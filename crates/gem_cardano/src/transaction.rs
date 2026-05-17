#[cfg(feature = "signer")]
use gem_hash::blake2::blake2b_256;

use crate::cbor::CborEncoder;

#[derive(Debug)]
pub(crate) struct TransactionInput {
    pub(crate) transaction_hash: [u8; 32],
    pub(crate) output_index: u64,
}

#[derive(Debug)]
pub(crate) struct TransactionOutput {
    pub(crate) address: Vec<u8>,
    pub(crate) amount: u64,
}

#[derive(Debug)]
pub(crate) struct Transaction {
    pub(crate) inputs: Vec<TransactionInput>,
    pub(crate) outputs: Vec<TransactionOutput>,
    pub(crate) fee: u64,
    pub(crate) expiration_block_number: u64,
}

impl Transaction {
    pub(crate) fn body_bytes(&self) -> Vec<u8> {
        let mut encoder = CborEncoder::new();
        encoder.map(4);

        encoder.unsigned(0);
        encoder.array(self.inputs.len());
        for input in &self.inputs {
            encoder.array(2);
            encoder.bytes(&input.transaction_hash);
            encoder.unsigned(input.output_index);
        }

        encoder.unsigned(1);
        encoder.array(self.outputs.len());
        for output in &self.outputs {
            encoder.array(2);
            encoder.bytes(&output.address);
            encoder.unsigned(output.amount);
        }

        encoder.unsigned(2);
        encoder.unsigned(self.fee);
        encoder.unsigned(3);
        encoder.unsigned(self.expiration_block_number);

        encoder.into_bytes()
    }

    #[cfg(feature = "signer")]
    pub(crate) fn transaction_id(&self) -> [u8; 32] {
        blake2b_256(&self.body_bytes())
    }

    pub(crate) fn signed_bytes(&self, public_key: &[u8; 32], signature: &[u8; 64]) -> Vec<u8> {
        let body = self.body_bytes();
        let mut encoder = CborEncoder::new();
        encoder.array(4);
        encoder.raw(&body);
        encoder.map(1);
        encoder.unsigned(0);
        encoder.array(1);
        encoder.array(2);
        encoder.bytes(public_key);
        encoder.bytes(signature);
        encoder.true_value();
        encoder.null();
        encoder.into_bytes()
    }

    pub(crate) fn signed_size(&self) -> usize {
        self.signed_bytes(&[0u8; 32], &[0u8; 64]).len()
    }
}

#[cfg(all(test, feature = "signer"))]
mod tests {
    use super::*;
    use crate::address::ShelleyAddress;

    #[test]
    fn test_transaction_encode_and_id() {
        let transaction = Transaction {
            inputs: vec![
                TransactionInput {
                    transaction_hash: hex::decode("f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e767").unwrap().try_into().unwrap(),
                    output_index: 1,
                },
                TransactionInput {
                    transaction_hash: hex::decode("554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af0").unwrap().try_into().unwrap(),
                    output_index: 0,
                },
            ],
            outputs: vec![
                TransactionOutput {
                    address: ShelleyAddress::parse("addr1q8043m5heeaydnvtmmkyuhe6qv5havvhsf0d26q3jygsspxlyfpyk6yqkw0yhtyvtr0flekj84u64az82cufmqn65zdsylzk23")
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                    amount: 2_000_000,
                },
                TransactionOutput {
                    address: ShelleyAddress::parse("addr1q92cmkgzv9h4e5q7mnrzsuxtgayvg4qr7y3gyx97ukmz3dfx7r9fu73vqn25377ke6r0xk97zw07dqr9y5myxlgadl2s0dgke5")
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                    amount: 16_749_189,
                },
            ],
            fee: 165_555,
            expiration_block_number: 53_333_345,
        };

        assert_eq!(
            hex::encode(transaction.body_bytes()),
            "a40082825820f074134aabbfb13b8aec7cf5465b1e5a862bde5cb88532cc7e64619179b3e76701825820554f2fd942a23d06835d26bbd78f0106fa94c8a551114a0bef81927f66467af000018282583901df58ee97ce7a46cd8bdeec4e5f3a03297eb197825ed5681191110804df22424b6880b39e4bac8c58de9fe6d23d79aaf44756389d827aa09b1a001e848082583901558dd902616f5cd01edcc62870cb4748c45403f1228218bee5b628b526f0ca9e7a2c04d548fbd6ce86f358be139fe680652536437d1d6fd51a00ff9285021a000286b3031a032dcd61"
        );
        assert_eq!(
            hex::encode(transaction.transaction_id()),
            "cc262713a3e15a0fa245b062f33ffc6c2aa5a64c3ae7bfa793414069914e1bbf"
        );
    }
}
