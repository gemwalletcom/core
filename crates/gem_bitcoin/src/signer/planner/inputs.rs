use std::str::FromStr;

use bitcoin::{Amount, OutPoint, Txid};
use primitives::{BitcoinChain, SignerError, UTXO};

use super::PlanInput;
use crate::signer::address::script_for_address;

const FINAL_SEQUENCE: u32 = 0xffff_ffff;
const RBF_SEQUENCE: u32 = 0xffff_fffd;

pub(super) fn spendable_inputs(chain: BitcoinChain, sender_address: &str, utxos: Vec<UTXO>, replace_by_fee: bool) -> Result<Vec<PlanInput>, SignerError> {
    let sender = script_for_address(chain, sender_address)?;
    let sender_hash = sender
        .unlocking_script()
        .zip(sender.public_key_hash())
        .map(|(_, public_key_hash)| public_key_hash)
        .ok_or_else(|| SignerError::invalid_input(format!("{} sender address type is unsupported", chain.get_chain())))?;
    let sequence = match chain {
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge if replace_by_fee => RBF_SEQUENCE,
        BitcoinChain::Bitcoin | BitcoinChain::BitcoinCash | BitcoinChain::Litecoin | BitcoinChain::Doge | BitcoinChain::Zcash => FINAL_SEQUENCE,
    };
    let mut inputs = Vec::with_capacity(utxos.len());

    for utxo in utxos {
        let address = script_for_address(chain, &utxo.address)?;
        let (unlocking_script, public_key_hash) = address
            .unlocking_script()
            .zip(address.public_key_hash())
            .ok_or_else(|| SignerError::invalid_input(format!("{} UTXO address type is unsupported", chain.get_chain())))?;
        (public_key_hash == sender_hash)
            .then_some(())
            .ok_or_else(|| SignerError::invalid_input(format!("{} UTXO address does not match sender address", chain.get_chain())))?;
        let value = utxo
            .value
            .parse::<u64>()
            .map_err(|_| SignerError::invalid_input(format!("invalid {} UTXO amount", chain.get_chain())))?;
        (value != 0)
            .then_some(())
            .ok_or_else(|| SignerError::invalid_input(format!("invalid {} UTXO amount", chain.get_chain())))?;
        let vout = u32::try_from(utxo.vout).map_err(|_| SignerError::invalid_input(format!("invalid {} UTXO output index", chain.get_chain())))?;
        let txid = Txid::from_str(&utxo.transaction_id).map_err(|_| SignerError::invalid_input(format!("invalid {} UTXO transaction id", chain.get_chain())))?;
        inputs.push(PlanInput {
            previous_output: OutPoint::new(txid, vout),
            value: Amount::from_sat(value),
            script_pubkey: address.script_pubkey,
            unlocking_script,
            sequence,
        });
    }

    Ok(inputs)
}

#[cfg(test)]
mod tests {
    use crate::{
        signer::address::UnlockingScript,
        testkit::{
            address_mock::{TEST_BITCOIN_P2WPKH_ADDRESS, TEST_BITCOIN_P2WPKH_HASH, prefixed_address},
            planner_mock::assert_invalid_input,
            signer_mock::utxo_with_address,
        },
    };

    use super::*;

    const TAPROOT_ADDRESS: &str = "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";

    #[test]
    fn test_spendable_inputs_address_type_validation() {
        let legacy_address = prefixed_address(&[0], TEST_BITCOIN_P2WPKH_HASH);
        let inputs = spendable_inputs(
            BitcoinChain::Bitcoin,
            TEST_BITCOIN_P2WPKH_ADDRESS,
            vec![utxo_with_address(&legacy_address), utxo_with_address(TEST_BITCOIN_P2WPKH_ADDRESS)],
            false,
        )
        .unwrap();
        assert_eq!(inputs[0].unlocking_script, UnlockingScript::P2pkh);
        assert_eq!(inputs[1].unlocking_script, UnlockingScript::P2wpkh);

        let different_legacy_address = prefixed_address(&[0], [9u8; 20]);
        assert_invalid_input(
            spendable_inputs(
                BitcoinChain::Bitcoin,
                TEST_BITCOIN_P2WPKH_ADDRESS,
                vec![utxo_with_address(&different_legacy_address)],
                false,
            ),
            "bitcoin UTXO address does not match sender address",
        );
        assert_invalid_input(
            spendable_inputs(BitcoinChain::Bitcoin, TEST_BITCOIN_P2WPKH_ADDRESS, vec![utxo_with_address(TAPROOT_ADDRESS)], false),
            "bitcoin UTXO address type is unsupported",
        );
        assert_invalid_input(
            spendable_inputs(BitcoinChain::Bitcoin, TAPROOT_ADDRESS, vec![utxo_with_address(TEST_BITCOIN_P2WPKH_ADDRESS)], false),
            "bitcoin sender address type is unsupported",
        );
    }
}
