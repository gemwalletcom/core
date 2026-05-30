pub(crate) mod address;
mod bitcoin_cash;
mod chain_signer;
mod encoding;
mod planner;
mod signature;
mod transaction;
mod types;
mod zcash;

#[cfg(feature = "rpc")]
use std::collections::HashMap;

#[cfg(feature = "rpc")]
use num_bigint::BigInt;
#[cfg(feature = "rpc")]
use primitives::{BitcoinChain, SignerError, SignerInput, TransactionFee, TransactionInputType, TransactionLoadInput};

pub use chain_signer::BitcoinChainSigner;
#[cfg(test)]
pub(crate) use planner::PlanInput;
pub use signature::sign_personal;
pub use types::{BitcoinSignDataResponse, BitcoinSignMessageData};

#[cfg(feature = "rpc")]
pub(crate) fn estimate_transaction_fee(chain: BitcoinChain, input: &TransactionLoadInput) -> Result<TransactionFee, SignerError> {
    let signer_input = SignerInput::new(input.clone(), input.default_fee());
    let request = match &input.input_type {
        TransactionInputType::Transfer(_) => planner::SpendRequest::transfer(chain, &signer_input, false)?,
        TransactionInputType::Swap(_, _, _) => planner::SpendRequest::swap(chain, &signer_input, false)?,
        _ => return SignerError::invalid_input_err("unsupported Bitcoin transaction type"),
    };
    let plan = planner::UtxoPlanner::plan(request)?;

    Ok(TransactionFee {
        fee: BigInt::from(plan.fee),
        gas_price_type: input.gas_price.clone(),
        gas_limit: BigInt::from(1u8),
        options: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use bitcoin::consensus::encode::deserialize;
    use primitives::{BitcoinChain, ChainSigner, SignerInput, SwapProvider, decode_hex};

    use super::{BitcoinChainSigner, address::script_for_address};
    use crate::testkit::signer_mock::{
        TEST_PRIVATE_KEY, contract_swap_input, contract_swap_input_with_provider, funded_transfer_input, p2wpkh_contract_swap_input, p2wpkh_transfer_input, transfer_input,
        transfer_swap_input,
    };

    const CHAINFLIP_NULLDATA_HEX: &str = "deadbeef001122";

    fn sign_transfer(chain: BitcoinChain) -> String {
        BitcoinChainSigner::new(chain).sign_transfer(&transfer_input(chain), &TEST_PRIVATE_KEY).unwrap()
    }

    fn assert_op_return_payload(script: &bitcoin::ScriptBuf, payload: &[u8]) {
        let bytes = script.as_bytes();
        assert_eq!(bytes[0], 0x6a);
        assert_eq!(bytes[1] as usize, payload.len());
        assert_eq!(&bytes[2..], payload);
    }

    fn sign_contract_swap(input: &SignerInput) -> bitcoin::Transaction {
        let raw = BitcoinChainSigner::new(BitcoinChain::Bitcoin).sign_swap(input, &TEST_PRIVATE_KEY).unwrap().remove(0);
        deserialize(&hex::decode(raw).unwrap()).unwrap()
    }

    fn sender_script(input: &SignerInput) -> bitcoin::ScriptBuf {
        script_for_address(BitcoinChain::Bitcoin, &input.sender_address).unwrap().script_pubkey
    }

    #[test]
    fn test_sign_transfer_bitcoin() {
        let raw = sign_transfer(BitcoinChain::Bitcoin);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(&raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x01);
    }

    #[test]
    fn test_sign_transfer_doge() {
        let input = funded_transfer_input(BitcoinChain::Doge);
        let raw = BitcoinChainSigner::new(BitcoinChain::Doge).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x01);
    }

    #[test]
    fn test_sign_transfer_bitcoin_cash() {
        let raw = sign_transfer(BitcoinChain::BitcoinCash);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let script = transaction.input[0].script_sig.as_bytes();
        let signature_len = script[0] as usize;
        assert_eq!(transaction.input.len(), 1);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(script[signature_len], 0x41);
    }

    #[test]
    fn test_signed_tx_is_rbf_signaled() {
        let raw = BitcoinChainSigner::new_with_rbf(BitcoinChain::Bitcoin, true)
            .sign_transfer(&transfer_input(BitcoinChain::Bitcoin), &TEST_PRIVATE_KEY)
            .unwrap();
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(&raw).unwrap()).unwrap();

        // RBF vector is checked against BitGoJS.
        assert_eq!(
            raw,
            "02000000010100000000000000000000000000000000000000000000000000000000000000000000006b483045022100ef21c70ee59cd7ef09f5d12252ed3c1c4fa971b33740cf2c29b90ab15fc3e5ea02205761ce83d839043d38341344c499cd13cb2ed4e03ebd88c6cf38fc84415b1e290121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078ffdffffff0210270000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac5e9b0000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000"
        );
        for input in &transaction.input {
            assert_eq!(input.sequence.0, 0xffff_fffd);
        }
    }

    #[test]
    fn test_sign_transfer_zcash() {
        let raw = sign_transfer(BitcoinChain::Zcash);
        let zcash_bytes = hex::decode(raw).unwrap();
        assert_eq!(&zcash_bytes[..20], hex::decode("050000800a27a726f04dec4d0000000000000000").unwrap().as_slice());
    }

    #[test]
    fn test_sign_transfer_vectors() {
        // Test vectors are checked against BitGoJS.
        let cases = [
            (
                "bitcoin_p2pkh",
                BitcoinChain::Bitcoin,
                transfer_input(BitcoinChain::Bitcoin),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006b483045022100f48a53c3e59e90789a4ecb96c5fdcd65f7bbbd8f4952e0f5a3ac276f9ad304ce0220160105ec967f69ef5546a08015481d0f2691b1a84ec2869c299e8538cce9ea3d0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac5e9b0000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "bitcoin_p2wpkh",
                BitcoinChain::Bitcoin,
                p2wpkh_transfer_input(),
                "0200000000010101000000000000000000000000000000000000000000000000000000000000000000000000ffffffff02102700000000000016001479b000887626b294a914501a4cd226b58b235983b39b00000000000016001479b000887626b294a914501a4cd226b58b23598302463043022036b08705576633f37dbb750b31140652e29985018b2a439b570c825d8c341870021f1fec61b2b2e0ba9a9996a7ac38358bf7280837b9801e1143f461fa905b6d540121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078f00000000",
            ),
            (
                "litecoin",
                BitcoinChain::Litecoin,
                transfer_input(BitcoinChain::Litecoin),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006a47304402207ffec22eb23eaa93b4959624320b71600bb56aaaee45ae214d5e1d864e47365802206844539bcf3eaaa32f5d95211e6caf3f1bc93a8a364befdd5b51aee4573f97f30121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914020202020202020202020202020202020202020288acd6970000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "doge",
                BitcoinChain::Doge,
                funded_transfer_input(BitcoinChain::Doge),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006a473044022076e879f42dcfcee85924d49aef75ecb3b7c50a50f6b9403b4a1192317580171902206f18320064953227d578ae0f073f55b454b1d1991b54058d9884d59ebb815f0b0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914020202020202020202020202020202020202020288ac2047f205000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "bitcoin_cash",
                BitcoinChain::BitcoinCash,
                transfer_input(BitcoinChain::BitcoinCash),
                "02000000010100000000000000000000000000000000000000000000000000000000000000000000006a47304402200433aef908cbbd4b65aec9dde055a94ba782bfb08a9451aa7bcdd10824345f3102207012a0573436b0d2e91a89b8cbd3f6c612bbcffb01a52d3fdeec9d51401ff7d34121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914020202020202020202020202020202020202020288acd6970000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac00000000",
            ),
            (
                "zcash",
                BitcoinChain::Zcash,
                transfer_input(BitcoinChain::Zcash),
                "050000800a27a726f04dec4d0000000000000000010100000000000000000000000000000000000000000000000000000000000000000000006a473044022004354fd389558909b1ccfb05dd2f3c324423efcad18eb50a42cb65ebdd17513d02207dac81ff91f4108b4017329e55971c2bd6a213cf925886b95513519bd806fd9a0121031b84c5567b126440995d3ed5aaba0565d71e1834604819ff9c17f5e9d5dd078fffffffff0210270000000000001976a914030303030303030303030303030303030303030388ac30750000000000001976a91479b000887626b294a914501a4cd226b58b23598388ac000000",
            ),
        ];

        for (name, chain, input, expected) in cases {
            let raw = BitcoinChainSigner::new(chain).sign_transfer(&input, &TEST_PRIVATE_KEY).unwrap();
            assert_eq!(raw, expected, "{name}");
        }
    }

    #[test]
    fn test_sign_swap_memo_op_return() {
        let memo = "=:s:0xEe7E9CcFb529f2c1Cc02C0Aea8aCed7Ec7e98B5e:0/1/0:g1:50";
        let input = transfer_swap_input(BitcoinChain::Doge, memo);
        let raw = BitcoinChainSigner::new(BitcoinChain::Doge).sign_swap(&input, &TEST_PRIVATE_KEY).unwrap().remove(0);
        let transaction: bitcoin::Transaction = deserialize(&hex::decode(raw).unwrap()).unwrap();
        let memo_output = transaction.output.iter().find(|output| output.script_pubkey.is_op_return()).unwrap();
        assert_eq!(memo_output.value.to_sat(), 0);
        assert_op_return_payload(&memo_output.script_pubkey, memo.as_bytes());
    }

    #[test]
    fn test_sign_chainflip_bitcoin_max_intent_produces_change_for_refund() {
        let nulldata = decode_hex(CHAINFLIP_NULLDATA_HEX).unwrap();
        let input = p2wpkh_contract_swap_input(CHAINFLIP_NULLDATA_HEX, true);
        let refund_script = sender_script(&input);
        let transaction = sign_contract_swap(&input);

        assert_eq!(transaction.output.len(), 3);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(transaction.output[1].value.to_sat(), 0);
        assert_op_return_payload(&transaction.output[1].script_pubkey, &nulldata);
        assert_eq!(transaction.output[2].value.to_sat(), 99_989_838);
        assert_eq!(transaction.output[2].script_pubkey, refund_script);
    }

    #[test]
    fn test_sign_chainflip_bitcoin_exact_swap_with_change() {
        let nulldata = decode_hex(CHAINFLIP_NULLDATA_HEX).unwrap();
        let input = contract_swap_input(BitcoinChain::Bitcoin, CHAINFLIP_NULLDATA_HEX, false);
        let transaction = sign_contract_swap(&input);

        assert_eq!(transaction.output.len(), 3);
        assert_eq!(transaction.output[0].value.to_sat(), 10_000);
        assert_eq!(transaction.output[1].value.to_sat(), 0);
        assert_op_return_payload(&transaction.output[1].script_pubkey, &nulldata);
        assert_eq!(transaction.output[2].value.to_sat(), 99_989_756);
    }

    #[test]
    fn test_chainflip_contract_swap_always_has_refund_output() {
        let nulldata = decode_hex(CHAINFLIP_NULLDATA_HEX).unwrap();
        for use_max_amount in [true, false] {
            let input = contract_swap_input(BitcoinChain::Bitcoin, CHAINFLIP_NULLDATA_HEX, use_max_amount);
            let refund_script = sender_script(&input);
            let transaction = sign_contract_swap(&input);

            assert_eq!(transaction.output.len(), 3, "{use_max_amount}");
            assert_eq!(transaction.output[1].value.to_sat(), 0, "{use_max_amount}");
            assert_op_return_payload(&transaction.output[1].script_pubkey, &nulldata);
            assert_eq!(transaction.output[2].script_pubkey, refund_script, "{use_max_amount}");
        }
    }

    #[test]
    fn test_non_chainflip_contract_swap_honors_max_flag() {
        let nulldata = decode_hex(CHAINFLIP_NULLDATA_HEX).unwrap();
        let input = contract_swap_input_with_provider(BitcoinChain::Bitcoin, CHAINFLIP_NULLDATA_HEX, true, SwapProvider::Thorchain);
        let transaction = sign_contract_swap(&input);

        assert_eq!(transaction.output.len(), 2);
        assert_eq!(transaction.output[0].value.to_sat(), 100_000_000 - 210);
        assert_eq!(transaction.output[1].value.to_sat(), 0);
        assert_op_return_payload(&transaction.output[1].script_pubkey, &nulldata);
    }
}
