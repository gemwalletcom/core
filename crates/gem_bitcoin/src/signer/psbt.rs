use std::collections::BTreeMap;

use bitcoin::{
    NetworkKind, PrivateKey, Psbt, PublicKey, Witness,
    bip32::{DerivationPath, Fingerprint, KeySource},
    secp256k1::Secp256k1,
};
use primitives::SignerError;

pub fn sign_psbt(psbt_hex: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let psbt_bytes = hex::decode(psbt_hex).map_err(|e| SignerError::invalid_input(format!("hex decode: {e}")))?;
    let mut psbt = Psbt::deserialize(&psbt_bytes).map_err(|e| SignerError::invalid_input(format!("psbt parse: {e}")))?;

    let secp = Secp256k1::new();
    let key = PrivateKey::from_slice(private_key, NetworkKind::Main).map_err(|e| SignerError::invalid_input(format!("private key: {e}")))?;
    let pub_key = PublicKey::from_private_key(&secp, &key);

    prepare_inputs(&mut psbt, &pub_key);
    sign(&mut psbt, &pub_key, &key, &secp)?;
    finalize_inputs(&mut psbt, &pub_key)?;

    let tx = psbt.extract_tx_unchecked_fee_rate();
    Ok(hex::encode(bitcoin::consensus::serialize(&tx)))
}

fn prepare_inputs(psbt: &mut Psbt, pub_key: &PublicKey) {
    let (x_only_key, _) = pub_key.inner.x_only_public_key();
    let default_origin: KeySource = (Fingerprint::default(), DerivationPath::master());

    for input in &mut psbt.inputs {
        let is_taproot = input.witness_utxo.as_ref().is_some_and(|utxo| utxo.script_pubkey.is_p2tr());

        if is_taproot {
            input.tap_internal_key.get_or_insert(x_only_key);
            if input.tap_key_origins.is_empty() {
                input.tap_key_origins.insert(x_only_key, (vec![], default_origin.clone()));
            }
        } else if input.bip32_derivation.is_empty() {
            input.bip32_derivation.insert(pub_key.inner, default_origin.clone());
        }
    }
}

fn sign(psbt: &mut Psbt, pub_key: &PublicKey, key: &PrivateKey, secp: &Secp256k1<bitcoin::secp256k1::All>) -> Result<(), SignerError> {
    let keys = BTreeMap::from([(*pub_key, *key)]);
    psbt.sign(&keys, secp).map(|_| ()).map_err(|(_ok, errors)| {
        let messages: Vec<String> = errors.into_iter().map(|(idx, e)| format!("input {idx}: {e}")).collect();
        SignerError::signing_error(messages.join(", "))
    })
}

fn finalize_inputs(psbt: &mut Psbt, pub_key: &PublicKey) -> Result<(), SignerError> {
    for (idx, input) in psbt.inputs.iter_mut().enumerate() {
        let script = &input
            .witness_utxo
            .as_ref()
            .ok_or_else(|| SignerError::signing_error(format!("missing witness_utxo for input {idx}")))?
            .script_pubkey;

        let witness = build_witness(input, pub_key, script, idx)?;
        input.final_script_witness = Some(witness);
        input.partial_sigs.clear();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation.clear();
    }
    Ok(())
}

fn build_witness(input: &bitcoin::psbt::Input, pub_key: &PublicKey, script: &bitcoin::ScriptBuf, idx: usize) -> Result<Witness, SignerError> {
    if script.is_p2wpkh() {
        let sig = input
            .partial_sigs
            .get(pub_key)
            .ok_or_else(|| SignerError::signing_error(format!("missing signature for input {idx}")))?;
        let mut w = Witness::new();
        w.push(sig.to_vec());
        w.push(pub_key.to_bytes());
        Ok(w)
    } else if script.is_p2tr() {
        let sig = input
            .tap_key_sig
            .ok_or_else(|| SignerError::signing_error(format!("missing taproot signature for input {idx}")))?;
        let mut w = Witness::new();
        w.push(sig.to_vec());
        Ok(w)
    } else {
        Err(SignerError::signing_error(format!("unsupported script type for input {idx}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TEST_PRIVATE_KEY;
    use bitcoin::{Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, hashes::Hash, secp256k1::Secp256k1};

    fn build_test_psbt(script_pubkey: ScriptBuf) -> Psbt {
        let utxo = TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: script_pubkey.clone(),
        };
        let tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Txid::all_zeros(), 0),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(90_000),
                script_pubkey,
            }],
        };
        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(utxo);
        psbt
    }

    fn sign_and_verify(psbt: Psbt) {
        let psbt_hex = hex::encode(psbt.serialize());
        let result = sign_psbt(&psbt_hex, &TEST_PRIVATE_KEY).unwrap();
        assert!(!result.is_empty());

        let tx: Transaction = bitcoin::consensus::deserialize(&hex::decode(&result).unwrap()).unwrap();
        assert_eq!(tx.input.len(), 1);
        assert!(!tx.input[0].witness.is_empty());
    }

    #[test]
    fn test_sign_p2wpkh_psbt() {
        let secp = Secp256k1::new();
        let key = PrivateKey::from_slice(&TEST_PRIVATE_KEY, NetworkKind::Main).unwrap();
        let pub_key = PublicKey::from_private_key(&secp, &key);
        let script = ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap());

        sign_and_verify(build_test_psbt(script));
    }

    #[test]
    fn test_sign_p2tr_psbt() {
        let secp = Secp256k1::new();
        let key = PrivateKey::from_slice(&TEST_PRIVATE_KEY, NetworkKind::Main).unwrap();
        let (x_only, _) = key.public_key(&secp).inner.x_only_public_key();
        let script = ScriptBuf::new_p2tr(&secp, x_only, None);

        sign_and_verify(build_test_psbt(script));
    }

    #[test]
    fn test_sign_psbt_invalid_hex() {
        assert!(sign_psbt("not_hex!", &TEST_PRIVATE_KEY).is_err());
    }

    #[test]
    fn test_sign_psbt_invalid_psbt() {
        assert!(sign_psbt("deadbeef", &TEST_PRIVATE_KEY).is_err());
    }
}
