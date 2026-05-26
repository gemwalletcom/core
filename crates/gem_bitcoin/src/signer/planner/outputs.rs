use bitcoin::{
    ScriptBuf,
    blockdata::{opcodes::all::OP_RETURN, script::Builder},
    script::PushBytesBuf,
};
use primitives::SignerError;

use super::PlanOutput;

const MAX_OP_RETURN_BYTES: usize = 80;

// OP_RETURN is placed at output index 1 (after the destination): required by Chainflip's
// vault-swap scanner, accepted by Thorchain which scans all outputs.
pub(super) fn spend_outputs(amount: u64, payment_script: ScriptBuf, memo_output: Option<PlanOutput>) -> Vec<PlanOutput> {
    let mut outputs = vec![PlanOutput::new(amount, payment_script)];
    if let Some(output) = memo_output {
        outputs.push(output);
    }
    outputs
}

pub(super) fn op_return_output(data: &[u8]) -> Result<PlanOutput, SignerError> {
    if data.len() > MAX_OP_RETURN_BYTES {
        return SignerError::invalid_input_err("Bitcoin memo is too large");
    }
    let push = PushBytesBuf::try_from(data.to_vec()).map_err(|_| SignerError::invalid_input("Bitcoin memo is too large"))?;
    Ok(PlanOutput::new(0, Builder::new().push_opcode(OP_RETURN).push_slice(push).into_script()))
}

pub(super) fn dust_threshold(script_pubkey: &ScriptBuf) -> u64 {
    if script_pubkey.is_op_return() {
        return 0;
    }
    546
}
