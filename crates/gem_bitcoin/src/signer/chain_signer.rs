use std::collections::BTreeMap;

use bitcoin::{
    NetworkKind, PrivateKey, Psbt, PublicKey, Witness,
    bip32::{DerivationPath, Fingerprint, KeySource},
    secp256k1::Secp256k1,
};
use primitives::{ChainSigner, SignerError, SwapProvider, TransactionLoadInput};

#[derive(Default)]
pub struct BitcoinChainSigner;

impl ChainSigner for BitcoinChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let provider = &swap_data.quote.provider_data.provider;

        match provider {
            SwapProvider::Relay => {
                let psbt_hex = &swap_data.data.data;
                let signed = sign_psbt(psbt_hex, private_key)?;
                Ok(vec![signed])
            }
            SwapProvider::Thorchain | SwapProvider::Chainflip => Err(SignerError::signing_error("bitcoin transfer swaps not yet implemented in Rust")),
            other => Err(SignerError::signing_error(format!("unsupported swap provider for Bitcoin: {:?}", other))),
        }
    }
}

fn sign_psbt(psbt_hex: &str, private_key: &[u8]) -> Result<String, SignerError> {
    let psbt_bytes = hex::decode(psbt_hex).map_err(|e| SignerError::invalid_input(format!("hex decode: {e}")))?;
    let psbt = Psbt::deserialize(&psbt_bytes).map_err(|e| SignerError::invalid_input(format!("psbt parse: {e}")))?;
    sign_and_finalize(psbt, private_key)
}

fn sign_and_finalize(mut psbt: Psbt, private_key: &[u8]) -> Result<String, SignerError> {
    let secp = Secp256k1::new();
    let key = PrivateKey::from_slice(private_key, NetworkKind::Main).map_err(|e| SignerError::invalid_input(format!("private key: {e}")))?;
    let pub_key = PublicKey::from_private_key(&secp, &key);
    let (x_only_key, _parity) = pub_key.inner.x_only_public_key();

    for input in &mut psbt.inputs {
        let is_taproot = input.witness_utxo.as_ref().is_some_and(|utxo| utxo.script_pubkey.is_p2tr());

        if is_taproot {
            if input.tap_internal_key.is_none() {
                input.tap_internal_key = Some(x_only_key);
            }
            if input.tap_key_origins.is_empty() {
                let key_source: KeySource = (Fingerprint::default(), DerivationPath::master());
                input.tap_key_origins.insert(x_only_key, (vec![], key_source));
            }
        } else if input.bip32_derivation.is_empty() {
            input.bip32_derivation.insert(pub_key.inner, (Fingerprint::default(), DerivationPath::master()));
        }
    }

    let mut keys = BTreeMap::new();
    keys.insert(pub_key, key);

    psbt.sign(&keys, &secp).map_err(|(_ok, errors)| {
        let messages: Vec<String> = errors.into_iter().map(|(idx, e)| format!("input {idx}: {e}")).collect();
        SignerError::signing_error(messages.join(", "))
    })?;

    finalize_inputs(&mut psbt, &pub_key)?;

    let transaction = psbt.extract_tx_unchecked_fee_rate();
    Ok(hex::encode(bitcoin::consensus::serialize(&transaction)))
}

fn finalize_inputs(psbt: &mut Psbt, pub_key: &PublicKey) -> Result<(), SignerError> {
    for (idx, input) in psbt.inputs.iter_mut().enumerate() {
        let script = input
            .witness_utxo
            .as_ref()
            .ok_or_else(|| SignerError::signing_error(format!("missing witness_utxo for input {idx}")))?
            .script_pubkey
            .clone();

        let witness = if script.is_p2wpkh() {
            let sig = input
                .partial_sigs
                .get(pub_key)
                .ok_or_else(|| SignerError::signing_error(format!("missing signature for input {idx}")))?;
            let mut w = Witness::new();
            w.push(sig.to_vec());
            w.push(pub_key.to_bytes());
            w
        } else if script.is_p2tr() {
            let sig = input
                .tap_key_sig
                .ok_or_else(|| SignerError::signing_error(format!("missing taproot signature for input {idx}")))?;
            let mut w = Witness::new();
            w.push(sig.to_vec());
            w
        } else {
            return Err(SignerError::signing_error(format!("unsupported script type for input {idx}")));
        };

        input.final_script_witness = Some(witness);
        input.partial_sigs = BTreeMap::new();
        input.sighash_type = None;
        input.redeem_script = None;
        input.witness_script = None;
        input.bip32_derivation = BTreeMap::new();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TEST_PRIVATE_KEY;
    use bitcoin::{
        Amount, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
        hashes::Hash,
        secp256k1::Secp256k1,
    };

    fn build_p2wpkh_psbt(pub_key: &PublicKey) -> Psbt {
        let wpkh = ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap());
        let utxo = TxOut { value: Amount::from_sat(100_000), script_pubkey: wpkh };

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
                script_pubkey: ScriptBuf::new_p2wpkh(&pub_key.wpubkey_hash().unwrap()),
            }],
        };

        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(utxo);
        psbt
    }

    fn build_p2tr_psbt(key: &PrivateKey) -> Psbt {
        let secp = Secp256k1::new();
        let (x_only, _) = key.public_key(&secp).inner.x_only_public_key();
        let script = ScriptBuf::new_p2tr(&secp, x_only, None);
        let utxo = TxOut { value: Amount::from_sat(100_000), script_pubkey: script.clone() };

        let tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: bitcoin::absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::new(Txid::all_zeros(), 0),
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut { value: Amount::from_sat(90_000), script_pubkey: script }],
        };

        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(utxo);
        psbt
    }

    #[test]
    fn test_sign_p2wpkh_psbt() {
        let secret = TEST_PRIVATE_KEY;
        let secp = Secp256k1::new();
        let key = PrivateKey::from_slice(&secret, NetworkKind::Main).unwrap();
        let pub_key = PublicKey::from_private_key(&secp, &key);

        let psbt = build_p2wpkh_psbt(&pub_key);
        let psbt_hex = hex::encode(psbt.serialize());

        let result = sign_psbt(&psbt_hex, &secret).unwrap();
        assert!(!result.is_empty());

        let tx_bytes = hex::decode(&result).unwrap();
        let tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes).unwrap();
        assert_eq!(tx.input.len(), 1);
        assert!(!tx.input[0].witness.is_empty());
    }

    #[test]
    fn test_sign_p2tr_psbt() {
        let secret = TEST_PRIVATE_KEY;
        let key = PrivateKey::from_slice(&secret, NetworkKind::Main).unwrap();

        let psbt = build_p2tr_psbt(&key);
        let psbt_hex = hex::encode(psbt.serialize());

        let result = sign_psbt(&psbt_hex, &secret).unwrap();
        assert!(!result.is_empty());

        let tx_bytes = hex::decode(&result).unwrap();
        let tx: Transaction = bitcoin::consensus::deserialize(&tx_bytes).unwrap();
        assert_eq!(tx.input.len(), 1);
        assert!(!tx.input[0].witness.is_empty());
    }

    #[test]
    fn test_sign_psbt_invalid_hex() {
        let result = sign_psbt("not_hex!", &TEST_PRIVATE_KEY);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_psbt_invalid_psbt() {
        let result = sign_psbt("deadbeef", &TEST_PRIVATE_KEY);
        assert!(result.is_err());
    }
}
