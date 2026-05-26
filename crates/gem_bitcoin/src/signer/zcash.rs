use bitcoin::{
    PublicKey, ScriptBuf, Sequence, TxIn as TransactionInput, TxOut as TransactionOutput, Witness,
    blockdata::script::Builder,
    consensus::encode::serialize,
    secp256k1::{Message, Secp256k1, SecretKey, Signing},
};
use gem_hash::blake2::blake2b_256_personal;
use primitives::{SignerError, TransactionLoadMetadata, decode_hex};

use crate::signer::{
    encoding::encode_varint,
    planner::{PlanInput, SpendPlan},
    transaction::{der_signature, signature_push_bytes},
};

const OVERWINTERED_VERSION_5: u32 = 0x8000_0005;
const VERSION_GROUP_ID_V5: u32 = 0x26a7_270a;
const LOCK_TIME: u32 = 0;
const EXPIRY_HEIGHT_DISABLED: u32 = 0;
const SIGHASH_ALL: u8 = 0x01;
const ZCASH_TRANSPARENT_AMOUNTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxTrAmountsHash";
const ZCASH_TRANSPARENT_SCRIPTS_HASH_PERSONALIZATION: &[u8; 16] = b"ZTxTrScriptsHash";

#[derive(Debug, Clone)]
struct ZcashTransparentTransaction {
    branch_id: u32,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

struct ZcashSignatureDigests {
    header: [u8; 32],
    prevouts: [u8; 32],
    amounts: [u8; 32],
    script_pubkeys: [u8; 32],
    sequences: [u8; 32],
    outputs: [u8; 32],
    sapling: [u8; 32],
    orchard: [u8; 32],
}

impl ZcashSignatureDigests {
    fn new(tx: &ZcashTransparentTransaction, plan: &SpendPlan) -> Result<Self, SignerError> {
        let prevouts = plan.inputs.iter().flat_map(|input| serialize(&input.previous_output)).collect::<Vec<_>>();
        let amounts = plan.inputs.iter().map(signed_value_bytes).collect::<Result<Vec<_>, _>>()?.concat();
        let script_pubkeys = plan.inputs.iter().flat_map(|input| serialize(input.script_pubkey.as_script())).collect::<Vec<_>>();
        let sequences = plan.inputs.iter().flat_map(|input| input.sequence.to_le_bytes()).collect::<Vec<_>>();
        let outputs = tx.outputs.iter().flat_map(serialize).collect::<Vec<_>>();

        Ok(Self {
            header: header_digest(tx.branch_id),
            prevouts: blake2b_256_personal(&prevouts, b"ZTxIdPrevoutHash"),
            amounts: blake2b_256_personal(&amounts, ZCASH_TRANSPARENT_AMOUNTS_HASH_PERSONALIZATION),
            script_pubkeys: blake2b_256_personal(&script_pubkeys, ZCASH_TRANSPARENT_SCRIPTS_HASH_PERSONALIZATION),
            sequences: blake2b_256_personal(&sequences, b"ZTxIdSequencHash"),
            outputs: blake2b_256_personal(&outputs, b"ZTxIdOutputsHash"),
            sapling: blake2b_256_personal(&[], b"ZTxIdSaplingHash"),
            orchard: blake2b_256_personal(&[], b"ZTxIdOrchardHash"),
        })
    }
}

impl ZcashTransparentTransaction {
    fn unsigned(plan: &SpendPlan, branch_id: u32) -> Self {
        let inputs = plan
            .inputs
            .iter()
            .map(|input| TransactionInput {
                previous_output: input.previous_output,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(input.sequence),
                witness: Witness::default(),
            })
            .collect();

        let outputs = plan
            .outputs
            .iter()
            .map(|output| TransactionOutput {
                value: output.value,
                script_pubkey: output.script_pubkey.clone(),
            })
            .collect();

        Self { branch_id, inputs, outputs }
    }

    fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&OVERWINTERED_VERSION_5.to_le_bytes());
        bytes.extend_from_slice(&VERSION_GROUP_ID_V5.to_le_bytes());
        bytes.extend_from_slice(&self.branch_id.to_le_bytes());
        bytes.extend_from_slice(&LOCK_TIME.to_le_bytes());
        bytes.extend_from_slice(&EXPIRY_HEIGHT_DISABLED.to_le_bytes());

        bytes.extend_from_slice(&encode_varint(self.inputs.len()));
        for input in &self.inputs {
            bytes.extend_from_slice(&serialize(input));
        }

        bytes.extend_from_slice(&encode_varint(self.outputs.len()));
        for output in &self.outputs {
            bytes.extend_from_slice(&serialize(output));
        }

        bytes.extend_from_slice(&encode_varint(0));
        bytes.extend_from_slice(&encode_varint(0));
        bytes.extend_from_slice(&encode_varint(0));
        bytes
    }
}

pub(crate) fn sign_transparent<C: Signing>(plan: &SpendPlan, branch_id: u32, secret_key: &SecretKey, public_key: &PublicKey, secp: &Secp256k1<C>) -> Result<String, SignerError> {
    let mut tx = ZcashTransparentTransaction::unsigned(plan, branch_id);
    let digests = ZcashSignatureDigests::new(&tx, plan)?;

    for (index, _) in plan.inputs.iter().enumerate() {
        let sighash = signature_digest(tx.branch_id, &digests, plan, index)?;
        let signature = der_signature(secp, secret_key, Message::from_digest(sighash), SIGHASH_ALL);
        tx.inputs[index].script_sig = Builder::new().push_slice(signature_push_bytes(signature)?).push_key(public_key).into_script();
    }

    Ok(hex::encode(tx.encode()))
}

pub(crate) fn branch_id_from_metadata(metadata: &TransactionLoadMetadata) -> Result<u32, SignerError> {
    let branch_id = metadata.get_branch_id().map_err(SignerError::from_display)?;
    let bytes: [u8; 4] = decode_hex(&branch_id)
        .map_err(SignerError::from)?
        .try_into()
        .map_err(|_| SignerError::invalid_input("invalid Zcash branch id"))?;
    // Blockbook reports consensus.chaintip in big-endian display order; Zcash
    // transaction encoding and personalization use the u32 in little-endian.
    Ok(u32::from_be_bytes(bytes))
}

fn signature_digest(branch_id: u32, digests: &ZcashSignatureDigests, plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
    let transparent_sig_digest = transparent_sig_digest(digests, plan, input_index)?;

    let mut bytes = Vec::with_capacity(32 * 4);
    bytes.extend_from_slice(&digests.header);
    bytes.extend_from_slice(&transparent_sig_digest);
    bytes.extend_from_slice(&digests.sapling);
    bytes.extend_from_slice(&digests.orchard);

    Ok(blake2b_256_personal(&bytes, &branch_personalization(b"ZcashTxHash_", branch_id)))
}

fn header_digest(branch_id: u32) -> [u8; 32] {
    let mut bytes = Vec::with_capacity(20);
    bytes.extend_from_slice(&OVERWINTERED_VERSION_5.to_le_bytes());
    bytes.extend_from_slice(&VERSION_GROUP_ID_V5.to_le_bytes());
    bytes.extend_from_slice(&branch_id.to_le_bytes());
    bytes.extend_from_slice(&LOCK_TIME.to_le_bytes());
    bytes.extend_from_slice(&EXPIRY_HEIGHT_DISABLED.to_le_bytes());
    blake2b_256_personal(&bytes, b"ZTxIdHeadersHash")
}

fn transparent_sig_digest(digests: &ZcashSignatureDigests, plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
    let mut bytes = Vec::with_capacity(1 + 32 * 6);
    bytes.push(SIGHASH_ALL);
    bytes.extend_from_slice(&digests.prevouts);
    bytes.extend_from_slice(&digests.amounts);
    bytes.extend_from_slice(&digests.script_pubkeys);
    bytes.extend_from_slice(&digests.sequences);
    bytes.extend_from_slice(&digests.outputs);
    bytes.extend_from_slice(&txin_sig_digest(plan, input_index)?);
    Ok(blake2b_256_personal(&bytes, b"ZTxIdTranspaHash"))
}

fn txin_sig_digest(plan: &SpendPlan, input_index: usize) -> Result<[u8; 32], SignerError> {
    let input = plan.inputs.get(input_index).ok_or_else(|| SignerError::signing_error("Zcash input index out of bounds"))?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&serialize(&input.previous_output));
    bytes.extend_from_slice(&signed_value_bytes(input)?);
    bytes.extend_from_slice(&serialize(input.script_pubkey.as_script()));
    bytes.extend_from_slice(&input.sequence.to_le_bytes());
    Ok(blake2b_256_personal(&bytes, b"Zcash___TxInHash"))
}

fn signed_value_bytes(input: &PlanInput) -> Result<[u8; 8], SignerError> {
    let value = i64::try_from(input.value.to_sat()).map_err(|_| SignerError::invalid_input("invalid Zcash UTXO amount"))?;
    Ok(value.to_le_bytes())
}

fn branch_personalization(prefix: &[u8; 12], branch_id: u32) -> [u8; 16] {
    let mut personal = [0u8; 16];
    personal[..12].copy_from_slice(prefix);
    personal[12..].copy_from_slice(&branch_id.to_le_bytes());
    personal
}

#[cfg(test)]
mod tests {
    use bitcoin::secp256k1::{Secp256k1, SecretKey};
    use primitives::{BitcoinChain, testkit::mock_zcash};

    use super::*;
    use crate::{
        signer::planner::{SpendRequest, UtxoPlanner},
        testkit::{
            address_mock::zcash_address,
            signer_mock::{TEST_PRIVATE_KEY, public_key, sender_address as test_sender_address},
        },
    };

    #[test]
    fn test_sign_transparent() {
        let public_key = public_key();
        let sender_address = test_sender_address(BitcoinChain::Zcash);
        let destination_address = zcash_address([2u8; 20]);
        let input = mock_zcash::signer_input(sender_address, destination_address);
        let request = SpendRequest::transfer(BitcoinChain::Zcash, &input, false).unwrap();
        let plan = UtxoPlanner::plan(request).unwrap();

        let branch_id = branch_id_from_metadata(&input.metadata).unwrap();
        let tx = ZcashTransparentTransaction::unsigned(&plan, branch_id);
        let digests = ZcashSignatureDigests::new(&tx, &plan).unwrap();
        assert_eq!(
            hex::encode(signature_digest(branch_id, &digests, &plan, 0).unwrap()),
            "0e9508ded3c1bbbf0a153622e1b5dee4303c33d45bcaa7fa1218cab57feeb065"
        );

        let raw = sign_transparent(
            &plan,
            branch_id,
            &SecretKey::from_slice(&TEST_PRIVATE_KEY).unwrap(),
            &public_key,
            &Secp256k1::signing_only(),
        )
        .unwrap();
        let bytes = hex::decode(raw).unwrap();
        assert_eq!(&bytes[..20], hex::decode("050000800a27a726f04dec4d0000000000000000").unwrap().as_slice());
        assert_eq!(plan.fee, 10_000);
        assert_eq!(bytes[20], 1);

        let script_len_index = 20 + 1 + 32 + 4;
        let script_len = bytes[script_len_index] as usize;
        let signature_len = bytes[script_len_index + 1] as usize;
        assert!(script_len > 0);
        assert_eq!(bytes[script_len_index + 2 + signature_len - 1], SIGHASH_ALL);
        assert_eq!(*bytes.last().unwrap(), 0);
    }
}
