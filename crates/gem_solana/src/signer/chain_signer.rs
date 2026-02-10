use base64::{Engine, engine::general_purpose::STANDARD};
use primitives::{ChainSigner, SignerError, TransactionInputType, TransactionLoadInput};
use rand::seq::IndexedRandom;
use solana_primitives::{
    Pubkey, VersionedTransaction,
    instructions::{program_ids::SYSTEM_PROGRAM_ID, system},
    sign_message,
};

use crate::models::jito::JITO_TIP_ACCOUNTS;

const SYSTEM_TRANSFER_DISCRIMINANT: u32 = 2;
const SYSTEM_TRANSFER_DATA_LEN: usize = 12; // 4-byte discriminant + 8-byte lamports

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let tx_base64 = match &input.input_type {
            TransactionInputType::Swap(_, _, swap_data) => &swap_data.data.data,
            _ => return Err(SignerError::invalid_input("expected swap transaction")),
        };

        let unit_price: u64 = input.gas_price.unit_price().to_string().parse().unwrap_or(0);
        let jito_tip = input.gas_price.jito_tip();

        let signed = Self::sign_transaction(tx_base64, private_key, unit_price, jito_tip, &input.sender_address)?;

        Ok(vec![signed])
    }
}

impl SolanaChainSigner {
    fn has_jito_tip(tx: &VersionedTransaction) -> bool {
        let sys_program = Pubkey::from_base58(SYSTEM_PROGRAM_ID).expect("invalid system program id");
        let sys_idx = match tx.account_keys().iter().position(|k| *k == sys_program) {
            Some(idx) => idx as u8,
            None => return false,
        };

        let jito_pubkeys: Vec<Pubkey> = JITO_TIP_ACCOUNTS.iter().filter_map(|addr| Pubkey::from_base58(addr).ok()).collect();

        let account_keys = tx.account_keys();
        for ix in tx.instructions() {
            if ix.program_id_index != sys_idx || ix.data.len() != SYSTEM_TRANSFER_DATA_LEN {
                continue;
            }
            if ix.data[0..4] != SYSTEM_TRANSFER_DISCRIMINANT.to_le_bytes() {
                continue;
            }
            if ix.accounts.len() >= 2 {
                let dest_idx = ix.accounts[1] as usize;
                if dest_idx < account_keys.len() && jito_pubkeys.contains(&account_keys[dest_idx]) {
                    return true;
                }
            }
        }
        false
    }

    fn sign_transaction(tx_base64: &str, private_key: &[u8], unit_price: u64, jito_tip: u64, sender_address: &str) -> Result<String, SignerError> {
        let data = STANDARD.decode(tx_base64).map_err(|e| SignerError::invalid_input(format!("base64 decode: {e}")))?;

        let mut tx = VersionedTransaction::deserialize_with_version(&data).map_err(|e| SignerError::invalid_input(format!("parse transaction: {e}")))?;

        // Skip message modifications if co-signers present — changing the message would invalidate their signatures
        if tx.signatures().len() <= 1 {
            if unit_price > 0 && tx.get_compute_unit_price().unwrap_or(0) < unit_price {
                tx.set_compute_unit_price(unit_price)
                    .map_err(|e| SignerError::invalid_input(format!("set compute unit price: {e}")))?;
            }

            // Jito tip only for legacy transactions — V0 tips are added server-side to avoid lookup table conflicts
            if jito_tip > 0 && matches!(tx, VersionedTransaction::Legacy { .. }) && !Self::has_jito_tip(&tx) {
                let from = Pubkey::from_base58(sender_address).map_err(|e| SignerError::invalid_input(format!("invalid sender: {e}")))?;
                let tip_account = JITO_TIP_ACCOUNTS
                    .choose(&mut rand::rng())
                    .ok_or_else(|| SignerError::invalid_input("no jito tip accounts"))?;
                let to = Pubkey::from_base58(tip_account).map_err(|e| SignerError::invalid_input(format!("invalid tip account: {e}")))?;
                tx.add_instruction(system::transfer(&from, &to, jito_tip))
                    .map_err(|e| SignerError::signing_error(format!("add jito tip: {e}")))?;
            }
        }

        let message_bytes = tx.serialize_message().map_err(|e| SignerError::signing_error(format!("serialize message: {e}")))?;

        let sig = sign_message(private_key, &message_bytes).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;

        let sigs = tx.signatures_mut();
        if sigs.is_empty() {
            sigs.push(sig);
        } else {
            sigs[0] = sig;
        }

        let bytes = tx.serialize().map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;

        Ok(STANDARD.encode(&bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_primitives::SignatureBytes;

    const MAYAN_V0_TX: &str = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQAFEYX7qT7inGBPqFijUWiMASkIQer7GcY6cKR108e8O++fsoWTLL6BMFk8QlX4RE70nGZIhhViOHEI/zWB6+akj+uKM/AFpmLQgvi3Uc9ORPKvU95f9YyeR2AGG38uRnJEhc9u0FRUiC4li6CDgDgrZl6r6UV4hTXUW6fWb5yNguwgdwR6OBw5FTj3o7pCuv6EHUU/JtUucaZkQ/avHt10iv1T73eq8oN5Kj5bBDHMNxcMebIKLnbq6WAyq7obsX2shOnUSIsH/jmbGpFV5YIbaX1DAWwKPE87vKKvtB0BYzBX9zrVjP00a2XVpGuL+frbPfhVHN7/gUcxBmXFi0Uh45wweXcehKgBk1IFjJqIXsEu//wlo69u6ByCkHnhP9jmcTziNUHzk/a6J1Lty1Ri5m2d13aoQy7RflktA4upsx58sf4CSerGwPpGg0r3OFO05QB53HvFOYx/Yh+VvbxQRRGI8f+jot/mF73E41cyUaMi4/yugeWkVzkOZHUcAKRl4gMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAtCPlnWlpkuqreRQnU8Cb1tgTWqlAT2NSRxIcSdKfQ5UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAR51VvyMcBu7nTFbs5oFQf9sbLeo/SOUQKxzaJWvBOPrBrj0IfykjcGJUj3DEwErsKplWlJhufLtGdSBiHThjDusasFT12pkjS9oh21wRb9zwNgcAPyhGx2BTuseVWqAgoMAAUCwFwVAAwACQMgoQcAAAAAAA0HGAABAiAOGQANBxgAAwAaDhkBAQ0DDgADDAIAAAAA4fUFAAAAAA0CGQMBEQ8pEAADBAUBGiAZGRsPIyQVEBYXBAYaHBkZHSEZGSIQESAcBRIGEwcICRQw0ZhTk3z+2OkFAOH1BQAAAABXTQYAAAAAAGQAAAAAAAIAAABZARAnAAEvAAAQJwECDQQZAwAAAQkNCh4AAAIBCiAfGQ7GASBMKQwnooTb00UGAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABRS8sfmqu5BOYQa9EFK2bScG27twIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB6pgUAAAAAAAAAAAAAAAAA1iUAAAAAAADjRgAAAAAAACbxiWkAAAAAAAAAAAAAAAAAAAAADZ2rGiSPY7CkiWW6hDXk3nSXo9wyAwK8aVtiEi4pTCKpF8dMTObcSxG7/d+dadNNUG/089Q4Sg4CAAsMAgAAABAnAAAAAAAAA9nDDfuWEY2uAzWl9dW04R5sD/Lws3UWLUj16RRlk9irAAgGBTA0ESohIJwNGt5f+UDYRx+qB4SLxIsIpumz2F+mL+NYMYuTMQyUBI6Ii48DiY2KeUBOLfg78dFJchk/vr91fbP9YECNHKxDH8G8DV2QZhYD298XAtwW";

    #[test]
    fn has_jito_tip_detects_tip_in_v0() {
        let data = STANDARD.decode(MAYAN_V0_TX).unwrap();
        let tx = VersionedTransaction::deserialize_with_version(&data).unwrap();
        assert!(SolanaChainSigner::has_jito_tip(&tx));
    }

    #[test]
    fn sign_transaction_roundtrips_v0() {
        let private_key = [1u8; 32];
        let result = SolanaChainSigner::sign_transaction(MAYAN_V0_TX, &private_key, 0, 0, "85fb693fba9c607a85a58b44259380012908416afb19c63a70a475d3c7bc3bef");
        assert!(result.is_ok(), "sign failed: {:?}", result.err());

        let signed_b64 = result.unwrap();
        let signed_bytes = STANDARD.decode(&signed_b64).unwrap();
        let signed_tx = VersionedTransaction::deserialize_with_version(&signed_bytes).unwrap();
        let sigs = signed_tx.signatures();
        assert!(!sigs.is_empty());
        assert_ne!(sigs[0], SignatureBytes::default());
    }
}
