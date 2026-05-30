use bitcoin::ScriptBuf;
use primitives::{BitcoinChain, SignerError};

use super::{
    PlanInput, PlanOutput, SpendPlan, SpendRequest,
    fee::estimate_fee,
    inputs::spendable_inputs,
    outputs::{dust_threshold, op_return_output, spend_outputs},
};
use crate::signer::address::script_for_address;

pub(crate) struct UtxoPlanner;

#[derive(Debug, Clone, Copy)]
enum SpendTarget {
    Exact(u64),
    Max,
}

impl UtxoPlanner {
    pub(crate) fn plan(request: SpendRequest) -> Result<SpendPlan, SignerError> {
        if request.utxos.is_empty() {
            return SignerError::invalid_input_err("missing input UTXOs");
        }
        if !request.is_max && request.amount == 0 {
            return SignerError::invalid_input_err("invalid transaction amount");
        }

        let payment_script = script_for_address(request.chain, &request.destination_address)?.script_pubkey;
        if !request.is_max && request.amount < dust_threshold(&payment_script) {
            return SignerError::invalid_input_err("invalid transaction amount");
        }

        let change_script = script_for_address(request.chain, &request.sender_address)?.script_pubkey;
        let memo_output = request.memo.as_deref().map(op_return_output).transpose()?;
        let spendable_inputs = spendable_inputs(request.chain, &request.sender_address, request.utxos, request.replace_by_fee)?;
        let target = if request.is_max { SpendTarget::Max } else { SpendTarget::Exact(request.amount) };
        Self::select_inputs_and_build_plan(request.chain, target, request.fee_rate, payment_script, change_script, memo_output, spendable_inputs)
    }

    fn select_inputs_and_build_plan(
        chain: BitcoinChain,
        target: SpendTarget,
        fee_rate: u64,
        payment_script: ScriptBuf,
        change_script: ScriptBuf,
        memo_output: Option<PlanOutput>,
        mut spendable_inputs: Vec<PlanInput>,
    ) -> Result<SpendPlan, SignerError> {
        match target {
            SpendTarget::Exact(_) => {
                spendable_inputs.sort_by(|left, right| {
                    left.value
                        .to_sat()
                        .cmp(&right.value.to_sat())
                        .then_with(|| left.previous_output.cmp(&right.previous_output))
                });
            }
            SpendTarget::Max => {}
        }

        let input_count = spendable_inputs.len();
        let mut selected = Vec::new();
        let mut selected_amount = 0u64;
        for (index, candidate) in spendable_inputs.into_iter().enumerate() {
            selected_amount = selected_amount
                .checked_add(candidate.value.to_sat())
                .ok_or_else(|| SignerError::invalid_input("Bitcoin amount overflow"))?;
            selected.push(candidate);
            match target {
                SpendTarget::Max if index + 1 < input_count => continue,
                SpendTarget::Max | SpendTarget::Exact(_) => {}
            }

            let plan = Self::try_build_plan(target, fee_rate, &payment_script, &change_script, &memo_output, &selected, selected_amount, chain)?;
            if let Some(plan) = plan {
                return Ok(plan);
            }
        }

        SignerError::invalid_input_err("insufficient balance")
    }

    fn try_build_plan(
        target: SpendTarget,
        fee_rate: u64,
        payment_script: &ScriptBuf,
        change_script: &ScriptBuf,
        memo_output: &Option<PlanOutput>,
        selected: &[PlanInput],
        selected_amount: u64,
        chain: BitcoinChain,
    ) -> Result<Option<SpendPlan>, SignerError> {
        let amount = match target {
            SpendTarget::Exact(amount) => amount,
            SpendTarget::Max => 0,
        };
        let mut outputs = spend_outputs(amount, payment_script.clone(), memo_output.clone());

        match target {
            SpendTarget::Max => {
                let fee = estimate_fee(chain, selected, &outputs, fee_rate)?;
                // Max-send planning is single-pass because BTC-family and ZIP-317 fees
                // depend on input/output scripts and counts, not the payment amount.
                let amount = selected_amount.checked_sub(fee).ok_or_else(|| SignerError::invalid_input("insufficient balance"))?;
                if amount == 0 {
                    return SignerError::invalid_input_err("insufficient balance");
                }
                if amount < dust_threshold(payment_script) {
                    return SignerError::invalid_input_err("insufficient balance");
                }

                outputs[0].value = bitcoin::Amount::from_sat(amount);
                return Ok(Some(SpendPlan {
                    inputs: selected.to_vec(),
                    outputs,
                    fee,
                }));
            }
            SpendTarget::Exact(_) => {}
        }

        let mut outputs_with_change = outputs.clone();
        outputs_with_change.push(PlanOutput::new(0, change_script.clone()));

        let fee_with_change = estimate_fee(chain, selected, &outputs_with_change, fee_rate)?;
        let Some(remainder) = selected_amount.checked_sub(amount).and_then(|value| value.checked_sub(fee_with_change)) else {
            return Ok(None);
        };

        if remainder >= dust_threshold(change_script) {
            outputs.push(PlanOutput::new(remainder, change_script.clone()));
            return Ok(Some(SpendPlan {
                inputs: selected.to_vec(),
                outputs,
                fee: fee_with_change,
            }));
        }

        let fee_without_change = estimate_fee(chain, selected, &outputs, fee_rate)?;
        let Some(remainder) = selected_amount.checked_sub(amount).and_then(|value| value.checked_sub(fee_without_change)) else {
            return Ok(None);
        };
        Ok(Some(SpendPlan {
            inputs: selected.to_vec(),
            outputs,
            fee: fee_without_change + remainder,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{
        address_mock::TEST_BITCOIN_P2WPKH_ADDRESS,
        planner_mock::{assert_invalid_input, spend_signer_input, spend_signer_input_with, spend_utxos, sum_inputs},
        signer_mock::{TEST_UTXO_TXID, utxo_with},
    };

    #[test]
    fn test_plan_transfer() {
        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("12000", false), false).unwrap();
        let plan = UtxoPlanner::plan(request).unwrap();
        assert_eq!(plan.inputs.len(), 2);
        assert_eq!(plan.outputs.len(), 3);
        assert_eq!(plan.outputs[0].value.to_sat(), 12_000);
        assert_eq!(plan.outputs[1].value.to_sat(), 0);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(
            sum_inputs(&plan.inputs).unwrap(),
            plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee
        );
        assert_eq!(plan.fee, 454);

        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("9300", false), false).unwrap();
        let plan = UtxoPlanner::plan(request).unwrap();
        assert_eq!(plan.inputs.len(), 1);
        assert_eq!(plan.outputs.len(), 2);
        assert_eq!(plan.outputs[0].value.to_sat(), 9_300);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(sum_inputs(&plan.inputs).unwrap(), 9_300 + plan.outputs[1].value.to_sat() + plan.fee);
        assert_eq!(plan.outputs[1].value.to_sat(), 0);

        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("50000", false), false).unwrap();
        assert_invalid_input(UtxoPlanner::plan(request), "insufficient balance");

        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("545", false), false).unwrap();
        assert_invalid_input(UtxoPlanner::plan(request), "invalid transaction amount");

        let dust_max_utxos = vec![utxo_with(TEST_UTXO_TXID, 0, "600", TEST_BITCOIN_P2WPKH_ADDRESS)];
        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input_with("0", true, None, dust_max_utxos), false).unwrap();
        assert_invalid_input(UtxoPlanner::plan(request), "insufficient balance");

        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input_with("1000", false, Some("a".repeat(81)), spend_utxos()), false).unwrap();
        assert_invalid_input(UtxoPlanner::plan(request), "Bitcoin memo is too large");

        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("0", true), false).unwrap();
        let plan = UtxoPlanner::plan(request).unwrap();
        assert_eq!(plan.inputs.len(), 2);
        assert_eq!(plan.outputs.len(), 2);
        assert!(plan.outputs[1].script_pubkey.is_op_return());
        assert_eq!(
            sum_inputs(&plan.inputs).unwrap(),
            plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee
        );
    }

    #[test]
    fn test_plan_absorbs_sub_dust_change_into_fee() {
        let request = SpendRequest::transfer(BitcoinChain::Bitcoin, &spend_signer_input("9300", false), false).unwrap();
        let change_script = script_for_address(request.chain, &request.sender_address).unwrap().script_pubkey;
        let plan = UtxoPlanner::plan(request).unwrap();

        assert_eq!(plan.outputs.len(), 2);
        assert_eq!(plan.outputs[0].value.to_sat(), 9_300);
        assert!(plan.outputs[1].script_pubkey.is_op_return());

        let selected_amount = sum_inputs(&plan.inputs).unwrap();
        let fee_without_change = estimate_fee(BitcoinChain::Bitcoin, &plan.inputs, &plan.outputs, 2).unwrap();
        let mut outputs_with_change = plan.outputs.clone();
        outputs_with_change.push(PlanOutput::new(0, change_script.clone()));
        let fee_with_change = estimate_fee(BitcoinChain::Bitcoin, &plan.inputs, &outputs_with_change, 2).unwrap();
        let dust_remainder = selected_amount - plan.outputs[0].value.to_sat() - fee_with_change;
        assert!(dust_remainder > 0);
        assert!(dust_remainder < dust_threshold(&change_script));

        let absorbed_remainder = selected_amount - plan.outputs[0].value.to_sat() - fee_without_change;
        assert_eq!(plan.fee, fee_without_change + absorbed_remainder);
        assert_eq!(selected_amount, plan.outputs.iter().map(|output| output.value.to_sat()).sum::<u64>() + plan.fee);
    }
}
