use bitcoin::{
    PublicKey, Transaction,
    blockdata::script::Builder,
    consensus::encode::serialize,
    secp256k1::{Message, Secp256k1, SecretKey, Signing},
};
use primitives::SignerError;

use crate::{
    hash::double_sha256,
    signer::{
        planner::SpendPlan,
        transaction::{build_unsigned_transaction, der_signature, signature_push_bytes},
    },
};

const SIGHASH_ALL_FORKID: u32 = 0x41;

struct SighashComponents {
    hash_prevouts: [u8; 32],
    hash_sequence: [u8; 32],
    hash_outputs: [u8; 32],
}

impl SighashComponents {
    fn new(tx: &Transaction, plan: &SpendPlan) -> Self {
        Self {
            hash_prevouts: double_sha256(&plan.inputs.iter().flat_map(|input| serialize(&input.previous_output)).collect::<Vec<_>>()),
            hash_sequence: double_sha256(&plan.inputs.iter().flat_map(|input| input.sequence.to_le_bytes()).collect::<Vec<_>>()),
            hash_outputs: double_sha256(&tx.output.iter().flat_map(serialize).collect::<Vec<_>>()),
        }
    }
}

pub(crate) fn sign_plan<C: Signing>(plan: &SpendPlan, secret_key: &SecretKey, public_key: &PublicKey, secp: &Secp256k1<C>) -> Result<Transaction, SignerError> {
    let mut tx = build_unsigned_transaction(plan);
    let components = SighashComponents::new(&tx, plan);
    for (index, _) in plan.inputs.iter().enumerate() {
        let sighash = signature_hash(&tx, plan, &components, index)?;
        let signature = der_signature(secp, secret_key, Message::from_digest(sighash), SIGHASH_ALL_FORKID as u8);
        tx.input[index].script_sig = Builder::new().push_slice(signature_push_bytes(signature)?).push_key(public_key).into_script();
    }
    Ok(tx)
}

fn signature_hash(tx: &Transaction, plan: &SpendPlan, components: &SighashComponents, input_index: usize) -> Result<[u8; 32], SignerError> {
    let input = plan
        .inputs
        .get(input_index)
        .ok_or_else(|| SignerError::signing_error("Bitcoin Cash input index out of bounds"))?;

    let mut preimage = Vec::new();
    preimage.extend_from_slice(&tx.version.0.to_le_bytes());
    preimage.extend_from_slice(&components.hash_prevouts);
    preimage.extend_from_slice(&components.hash_sequence);
    preimage.extend_from_slice(&serialize(&input.previous_output));
    preimage.extend_from_slice(&serialize(input.script_pubkey.as_script()));
    preimage.extend_from_slice(&input.value.to_sat().to_le_bytes());
    preimage.extend_from_slice(&input.sequence.to_le_bytes());
    preimage.extend_from_slice(&components.hash_outputs);
    preimage.extend_from_slice(&tx.lock_time.to_consensus_u32().to_le_bytes());
    preimage.extend_from_slice(&SIGHASH_ALL_FORKID.to_le_bytes());

    Ok(double_sha256(&preimage))
}
