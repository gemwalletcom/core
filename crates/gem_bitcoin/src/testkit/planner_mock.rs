use bitcoin::{
    Amount, OutPoint, ScriptBuf,
    blockdata::{opcodes::all::OP_RETURN, script::Builder},
    script::PushBytesBuf,
};
use num_bigint::BigInt;
use primitives::{BitcoinChain, GasPriceType, SignerError, SignerInput, TransactionFee, UTXO};

use crate::{
    signer::{PlanInput, address::UnlockingScript},
    testkit::{
        address_mock::TEST_BITCOIN_P2WPKH_ADDRESS,
        signer_mock::{TEST_UTXO_TXID, transfer_input_with_utxos, utxo_with},
    },
};

pub(crate) const TEST_SPEND_RECIPIENT: &str = "1BoatSLRHtKNngkdXEeobR76b53LETtpyT";

impl PlanInput {
    pub(crate) fn mock_with_unlocking_script(unlocking_script: UnlockingScript) -> Self {
        Self {
            previous_output: OutPoint::null(),
            value: Amount::from_sat(50_000),
            script_pubkey: ScriptBuf::new(),
            unlocking_script,
            sequence: u32::MAX,
        }
    }
}

pub(crate) fn spend_signer_input(value: &str, is_max: bool) -> SignerInput {
    spend_signer_input_with(value, is_max, Some("memo".to_string()), spend_utxos())
}

pub(crate) fn spend_signer_input_with(value: &str, is_max: bool, memo: Option<String>, utxos: Vec<UTXO>) -> SignerInput {
    let mut input = transfer_input_with_utxos(BitcoinChain::Bitcoin, TEST_BITCOIN_P2WPKH_ADDRESS, TEST_SPEND_RECIPIENT, value, utxos);
    input.input.gas_price = GasPriceType::regular(BigInt::from(2u64));
    input.input.memo = memo;
    input.input.is_max_value = is_max;
    input.fee = TransactionFee::new_from_fee(BigInt::from(2u64));
    input
}

pub(crate) fn spend_utxos() -> Vec<UTXO> {
    vec![
        utxo_with(TEST_UTXO_TXID, 0, "10000", TEST_BITCOIN_P2WPKH_ADDRESS),
        utxo_with("0000000000000000000000000000000000000000000000000000000000000002", 1, "20000", TEST_BITCOIN_P2WPKH_ADDRESS),
    ]
}

pub(crate) fn op_return_script(bytes: usize) -> ScriptBuf {
    let push = PushBytesBuf::try_from(vec![0u8; bytes]).unwrap();
    Builder::new().push_opcode(OP_RETURN).push_slice(push).into_script()
}

pub(crate) fn sum_inputs(inputs: &[PlanInput]) -> Result<u64, SignerError> {
    inputs.iter().try_fold(0u64, |sum, input| {
        sum.checked_add(input.value.to_sat()).ok_or_else(|| SignerError::invalid_input("Bitcoin amount overflow"))
    })
}

pub(crate) fn assert_invalid_input<T>(result: Result<T, SignerError>, expected: &str) {
    match result {
        Err(SignerError::InvalidInput(message)) => assert_eq!(message, expected),
        Err(SignerError::SigningError(message)) => panic!("unexpected signing error: {message}"),
        Ok(_) => panic!("expected invalid input error"),
    }
}
