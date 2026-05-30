use bitcoin::{Amount, OutPoint, ScriptBuf};

use crate::signer::{address::UnlockingScript, encoding::varint_len};

#[derive(Debug, Clone)]
pub(crate) struct PlanInput {
    pub(crate) previous_output: OutPoint,
    pub(crate) value: Amount,
    pub(crate) script_pubkey: ScriptBuf,
    pub(crate) unlocking_script: UnlockingScript,
    pub(crate) sequence: u32,
}

#[derive(Debug, Clone)]
pub(crate) struct PlanOutput {
    pub(crate) value: Amount,
    pub(crate) script_pubkey: ScriptBuf,
}

impl PlanOutput {
    pub(crate) fn new(value: u64, script_pubkey: ScriptBuf) -> Self {
        Self {
            value: Amount::from_sat(value),
            script_pubkey,
        }
    }

    pub(crate) fn serialized_len(&self) -> u64 {
        8 + varint_len(self.script_pubkey.len()) as u64 + self.script_pubkey.len() as u64
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SpendPlan {
    pub(crate) inputs: Vec<PlanInput>,
    pub(crate) outputs: Vec<PlanOutput>,
    pub(crate) fee: u64,
}
