use primitives::{BitcoinChain, SignerError};

use super::{PlanInput, PlanOutput};
use crate::signer::{address::UnlockingScript, encoding::varint_len};

const WITNESS_SCALE_FACTOR: u64 = 4;
const TX_FIXED_BYTES: u64 = 8;
const SEGWIT_MARKER_FLAG_WEIGHT: u64 = 2;
const P2PKH_INPUT_BYTES: u64 = 148;
const P2WPKH_INPUT_BASE_BYTES: u64 = 41;
const P2WPKH_INPUT_WITNESS_BYTES: u64 = 108;

pub(super) fn estimate_fee(chain: BitcoinChain, inputs: &[PlanInput], outputs: &[PlanOutput], fee_rate: u64) -> Result<u64, SignerError> {
    match chain {
        BitcoinChain::Zcash => return estimate_zcash_fee(inputs, outputs),
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge => {}
    }

    let has_witness = inputs.iter().any(|input| match input.unlocking_script {
        UnlockingScript::P2wpkh => true,
        UnlockingScript::P2pkh => false,
    });
    let input_weight = inputs.iter().try_fold(0u64, |sum, input| {
        sum.checked_add(input_weight(input)).ok_or_else(|| SignerError::invalid_input("Bitcoin fee overflow"))
    })?;
    let output_weight = outputs.iter().try_fold(0u64, |sum, output| {
        sum.checked_add(output.serialized_len() * WITNESS_SCALE_FACTOR)
            .ok_or_else(|| SignerError::invalid_input("Bitcoin fee overflow"))
    })?;
    transaction_base_weight(inputs.len(), outputs.len(), has_witness)?
        .checked_add(input_weight)
        .and_then(|value| value.checked_add(output_weight))
        .map(|weight| weight.div_ceil(WITNESS_SCALE_FACTOR))
        .and_then(|vbytes| vbytes.checked_mul(fee_rate))
        .ok_or_else(|| SignerError::invalid_input("Bitcoin fee overflow"))
}

fn transaction_base_weight(input_count: usize, output_count: usize, has_witness: bool) -> Result<u64, SignerError> {
    let base_bytes = TX_FIXED_BYTES
        .checked_add(varint_len(input_count) as u64)
        .and_then(|value| value.checked_add(varint_len(output_count) as u64))
        .ok_or_else(|| SignerError::invalid_input("Bitcoin fee overflow"))?;
    let witness_weight = if has_witness { SEGWIT_MARKER_FLAG_WEIGHT } else { 0 };
    base_bytes
        .checked_mul(WITNESS_SCALE_FACTOR)
        .and_then(|value| value.checked_add(witness_weight))
        .ok_or_else(|| SignerError::invalid_input("Bitcoin fee overflow"))
}

fn input_weight(input: &PlanInput) -> u64 {
    match input.unlocking_script {
        UnlockingScript::P2pkh => P2PKH_INPUT_BYTES * WITNESS_SCALE_FACTOR,
        UnlockingScript::P2wpkh => P2WPKH_INPUT_BASE_BYTES * WITNESS_SCALE_FACTOR + P2WPKH_INPUT_WITNESS_BYTES,
    }
}

fn estimate_zcash_fee(inputs: &[PlanInput], outputs: &[PlanOutput]) -> Result<u64, SignerError> {
    const MARGINAL_FEE: u64 = 5_000;
    const GRACE_ACTIONS: u64 = 2;
    const P2PKH_STANDARD_INPUT_SIZE: u64 = 150;
    const P2PKH_STANDARD_OUTPUT_SIZE: u64 = 34;

    let tx_in_total_size = inputs
        .len()
        .checked_mul(P2PKH_STANDARD_INPUT_SIZE as usize)
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| SignerError::invalid_input("Zcash fee overflow"))?;
    let tx_out_total_size = outputs.iter().try_fold(0u64, |sum, output| {
        sum.checked_add(output.serialized_len()).ok_or_else(|| SignerError::invalid_input("Zcash fee overflow"))
    })?;
    let input_actions = tx_in_total_size.div_ceil(P2PKH_STANDARD_INPUT_SIZE);
    let output_actions = tx_out_total_size.div_ceil(P2PKH_STANDARD_OUTPUT_SIZE);
    let logical_actions = input_actions.max(output_actions);
    MARGINAL_FEE
        .checked_mul(GRACE_ACTIONS.max(logical_actions))
        .ok_or_else(|| SignerError::invalid_input("Zcash fee overflow"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{signer::address::script_for_public_key_hash, testkit::planner_mock::op_return_script};

    #[test]
    fn test_estimate_fee() {
        let legacy_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh)];
        let legacy_outputs = vec![
            PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
            PlanOutput::new(20_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20])),
        ];
        assert_eq!(estimate_fee(BitcoinChain::Bitcoin, &legacy_inputs, &legacy_outputs, 2).unwrap(), 452);

        let segwit_inputs = vec![PlanInput::mock_with_unlocking_script(UnlockingScript::P2wpkh)];
        let segwit_outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2wpkh, [0u8; 20]))];
        assert_eq!(estimate_fee(BitcoinChain::Bitcoin, &segwit_inputs, &segwit_outputs, 3).unwrap(), 330);

        let zcash_outputs = vec![PlanOutput::new(10_000, script_for_public_key_hash(UnlockingScript::P2pkh, [0u8; 20]))];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &legacy_inputs, &zcash_outputs, 100).unwrap(), 10_000);

        let zcash_inputs = vec![
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
            PlanInput::mock_with_unlocking_script(UnlockingScript::P2pkh),
        ];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &zcash_inputs, &zcash_outputs, 100).unwrap(), 15_000);

        let zcash_large_output = vec![PlanOutput::new(0, op_return_script(80))];
        assert_eq!(estimate_fee(BitcoinChain::Zcash, &legacy_inputs, &zcash_large_output, 100).unwrap(), 15_000);
    }
}
