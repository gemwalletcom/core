use bitcoin::{
    PublicKey, ScriptBuf, Sequence, Transaction, TxIn as TransactionInput, TxOut as TransactionOutput, Witness,
    absolute::LockTime,
    blockdata::{script::Builder, transaction::Version},
    consensus::encode::serialize,
    script::PushBytesBuf,
    secp256k1::{Message, PublicKey as Secp256k1PublicKey, Secp256k1, SecretKey, Signing},
    sighash::{EcdsaSighashType, SighashCache},
};
use primitives::{BitcoinChain, SignerError};

use crate::signer::{
    address::{UnlockingScript, public_key_hash, script_for_public_key_hash},
    bitcoin_cash::sign_plan as sign_bitcoin_cash,
    planner::SpendPlan,
    zcash::sign_transparent,
};

pub(crate) fn sign_plan(chain: BitcoinChain, plan: &SpendPlan, private_key: &[u8], zcash_branch_id: Option<u32>) -> Result<String, SignerError> {
    let secret_key = SecretKey::from_slice(private_key).map_err(|_| SignerError::invalid_input(format!("invalid {} private key", chain.get_chain())))?;
    let secp = Secp256k1::signing_only();
    let public_key = PublicKey::new(Secp256k1PublicKey::from_secret_key(&secp, &secret_key));
    validate_chain_input_types(chain, plan)?;
    validate_public_key(chain, plan, &public_key)?;
    validate_plan_amounts(chain, plan)?;

    let tx = match chain {
        BitcoinChain::BitcoinCash => sign_bitcoin_cash(plan, &secret_key, &public_key, &secp)?,
        BitcoinChain::Bitcoin | BitcoinChain::Litecoin | BitcoinChain::Doge => sign_bitcoin_like(plan, &secret_key, &public_key, &secp)?,
        BitcoinChain::Zcash => {
            let branch_id = zcash_branch_id.ok_or_else(|| SignerError::invalid_input("missing Zcash branch id"))?;
            return sign_transparent(plan, branch_id, &secret_key, &public_key, &secp);
        }
    };

    Ok(hex::encode(serialize(&tx)))
}

pub(super) fn build_unsigned_transaction(plan: &SpendPlan) -> Transaction {
    let input = plan
        .inputs
        .iter()
        .map(|input| TransactionInput {
            previous_output: input.previous_output,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(input.sequence),
            witness: Witness::default(),
        })
        .collect();

    let output = plan
        .outputs
        .iter()
        .map(|output| TransactionOutput {
            value: output.value,
            script_pubkey: output.script_pubkey.clone(),
        })
        .collect();

    Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input,
        output,
    }
}

fn sign_bitcoin_like<C: Signing>(plan: &SpendPlan, secret_key: &SecretKey, public_key: &PublicKey, secp: &Secp256k1<C>) -> Result<Transaction, SignerError> {
    let mut tx = build_unsigned_transaction(plan);
    let signed_inputs = {
        let mut sighash_cache = SighashCache::new(&tx);
        let mut signed_inputs = Vec::with_capacity(plan.inputs.len());

        for (index, input) in plan.inputs.iter().enumerate() {
            match input.unlocking_script {
                UnlockingScript::P2pkh => {
                    let sighash = sighash_cache
                        .legacy_signature_hash(index, &input.script_pubkey, EcdsaSighashType::All.to_u32())
                        .map_err(|_| SignerError::signing_error("failed to compute Bitcoin sighash"))?;
                    let signature = der_signature(secp, secret_key, Message::from(sighash), EcdsaSighashType::All.to_u32() as u8);
                    let script_sig = Builder::new().push_slice(signature_push_bytes(signature)?).push_key(public_key).into_script();
                    signed_inputs.push((script_sig, Witness::default()));
                }
                UnlockingScript::P2wpkh => {
                    let sighash = sighash_cache
                        .p2wpkh_signature_hash(index, &input.script_pubkey, input.value, EcdsaSighashType::All)
                        .map_err(|_| SignerError::signing_error("failed to compute Bitcoin witness sighash"))?;
                    let signature = der_signature(secp, secret_key, Message::from(sighash), EcdsaSighashType::All.to_u32() as u8);
                    let mut witness = Witness::default();
                    witness.push(signature);
                    witness.push(public_key.to_bytes());
                    signed_inputs.push((ScriptBuf::new(), witness));
                }
            }
        }
        signed_inputs
    };

    for (input, (script_sig, witness)) in tx.input.iter_mut().zip(signed_inputs) {
        input.script_sig = script_sig;
        input.witness = witness;
    }

    Ok(tx)
}

fn validate_chain_input_types(chain: BitcoinChain, plan: &SpendPlan) -> Result<(), SignerError> {
    for input in &plan.inputs {
        match chain {
            BitcoinChain::Bitcoin | BitcoinChain::Litecoin | BitcoinChain::Doge => {}
            BitcoinChain::BitcoinCash | BitcoinChain::Zcash => {
                (input.unlocking_script == UnlockingScript::P2pkh)
                    .then_some(())
                    .ok_or_else(|| SignerError::invalid_input(format!("{} UTXO address type is unsupported", chain.get_chain())))?;
            }
        }
    }
    Ok(())
}

fn validate_public_key(chain: BitcoinChain, plan: &SpendPlan, public_key: &PublicKey) -> Result<(), SignerError> {
    let key_hash = public_key_hash(&public_key.to_bytes());
    for input in &plan.inputs {
        let expected = script_for_public_key_hash(input.unlocking_script, key_hash);
        (expected == input.script_pubkey)
            .then_some(())
            .ok_or_else(|| SignerError::invalid_input(format!("{} private key does not match sender address", chain.get_chain())))?;
    }
    Ok(())
}

fn validate_plan_amounts(chain: BitcoinChain, plan: &SpendPlan) -> Result<(), SignerError> {
    let input_total = plan.inputs.iter().try_fold(0u64, |sum, input| {
        sum.checked_add(input.value.to_sat())
            .ok_or_else(|| SignerError::invalid_input(format!("{} amount overflow", chain.get_chain())))
    })?;
    let output_total = plan.outputs.iter().try_fold(0u64, |sum, output| {
        sum.checked_add(output.value.to_sat())
            .ok_or_else(|| SignerError::invalid_input(format!("{} amount overflow", chain.get_chain())))
    })?;
    let spent_total = output_total
        .checked_add(plan.fee)
        .ok_or_else(|| SignerError::invalid_input(format!("{} amount overflow", chain.get_chain())))?;
    (input_total == spent_total).then_some(()).ok_or_else(|| {
        SignerError::invalid_input(format!(
            "{} plan amount mismatch: inputs {}, outputs {}, fee {}",
            chain.get_chain(),
            input_total,
            output_total,
            plan.fee
        ))
    })?;
    Ok(())
}

pub(super) fn der_signature<C: Signing>(secp: &Secp256k1<C>, secret_key: &SecretKey, message: Message, sighash_byte: u8) -> Vec<u8> {
    let signature = secp.sign_ecdsa(&message, secret_key);
    let mut bytes = signature.serialize_der().to_vec();
    bytes.push(sighash_byte);
    bytes
}

pub(super) fn signature_push_bytes(signature: Vec<u8>) -> Result<PushBytesBuf, SignerError> {
    PushBytesBuf::try_from(signature).map_err(|_| SignerError::signing_error("invalid Bitcoin script push"))
}
