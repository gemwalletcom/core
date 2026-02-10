use base64::{Engine, engine::general_purpose::STANDARD};
use num_traits::ToPrimitive;
use primitives::{ChainSigner, SignerError, TransactionLoadInput};
use solana_primitives::{
    Instruction, Pubkey, VersionedTransaction,
    instructions::{program_ids, system},
    sign_message,
};

use crate::models::jito;

const SYSTEM_TRANSFER_DISCRIMINANT: u32 = 2;
const SYSTEM_TRANSFER_DATA_LEN: usize = 12; // 4-byte discriminant + 8-byte lamports

#[derive(Default)]
pub struct SolanaChainSigner;

impl ChainSigner for SolanaChainSigner {
    fn sign_swap(&self, input: &TransactionLoadInput, private_key: &[u8]) -> Result<Vec<String>, SignerError> {
        let swap_data = input.input_type.get_swap_data().map_err(SignerError::invalid_input)?;
        let tx_base64 = &swap_data.data.data;

        let unit_price: u64 = input.gas_price.unit_price().to_u64().unwrap_or(0);
        let jito_tip = input.gas_price.jito_tip();

        let signed = Self::sign_transaction(tx_base64, private_key, unit_price, jito_tip, &input.sender_address)?;

        Ok(vec![signed])
    }
}

impl SolanaChainSigner {
    fn create_jito_tip_instruction(sender_address: &str, lamports: u64) -> Result<Instruction, SignerError> {
        let from = Pubkey::from_base58(sender_address).map_err(|e| SignerError::invalid_input(format!("invalid sender: {e}")))?;
        let to = jito::random_tip_pubkey();
        Ok(system::transfer(&from, &to, lamports))
    }

    fn has_jito_tip(transaction: &VersionedTransaction) -> bool {
        let sys_program = program_ids::system_program();
        let sys_idx = match transaction.account_keys().iter().position(|k| *k == sys_program) {
            Some(idx) => idx as u8,
            None => return false,
        };

        let jito_pubkeys: Vec<Pubkey> = jito::JITO_TIP_ACCOUNTS.iter().filter_map(|addr| Pubkey::from_base58(addr).ok()).collect();

        let account_keys = transaction.account_keys();
        for instruction in transaction.instructions() {
            if instruction.program_id_index != sys_idx || instruction.data.len() != SYSTEM_TRANSFER_DATA_LEN {
                continue;
            }
            if instruction.data[0..4] != SYSTEM_TRANSFER_DISCRIMINANT.to_le_bytes() {
                continue;
            }
            if instruction.accounts.len() >= 2 {
                let dest_idx = instruction.accounts[1] as usize;
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
                let tip_ix = Self::create_jito_tip_instruction(sender_address, jito_tip)?;
                tx.add_instruction(tip_ix).map_err(|e| SignerError::signing_error(format!("add jito tip: {e}")))?;
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
}
