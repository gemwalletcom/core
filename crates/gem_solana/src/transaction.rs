use gem_encoding::{decode_base64, encode_base64};
use primitives::SolanaInstruction;
use solana_primitives::{AccountMeta, AddressLookupTableAccount, Instruction, Pubkey, TransactionBuilder, VersionedTransaction};

pub fn try_decode_transaction(transaction_base64: &str) -> Option<VersionedTransaction> {
    let data = decode_base64(transaction_base64).ok()?;
    try_decode_transaction_bytes(&data)
}

pub(crate) fn try_decode_transaction_bytes(transaction: &[u8]) -> Option<VersionedTransaction> {
    let decoded = VersionedTransaction::deserialize_with_version(transaction).ok()?;
    // Ensure the parser consumed exactly this transaction, with no trailing bytes.
    (decoded.serialize().ok()? == transaction).then_some(decoded)
}

pub(crate) fn is_transaction_bytes(transaction: &[u8]) -> bool {
    try_decode_transaction_bytes(transaction).is_some() || try_decode_transaction_message(transaction).is_some()
}

fn try_decode_transaction_message(message: &[u8]) -> Option<VersionedTransaction> {
    let mut transaction = Vec::with_capacity(message.len() + 1);
    transaction.push(0);
    transaction.extend_from_slice(message);

    let decoded = VersionedTransaction::deserialize_with_version(&transaction).ok()?;
    (decoded.serialize_message().ok()? == message).then_some(decoded)
}

pub fn decode_transaction(transaction_base64: &str) -> Result<VersionedTransaction, String> {
    try_decode_transaction(transaction_base64).ok_or_else(|| "failed to decode transaction".to_string())
}

pub fn try_decode_blockhash(blockhash: &str) -> Option<[u8; 32]> {
    bs58::decode(blockhash).into_vec().ok()?.try_into().ok()
}

pub fn encode_v0_transaction(payer: Pubkey, recent_blockhash: &str, instructions: &[Instruction], lookup_tables: &[AddressLookupTableAccount]) -> Result<String, String> {
    let recent_blockhash = try_decode_blockhash(recent_blockhash).ok_or_else(|| "Invalid Solana blockhash".to_string())?;
    let transaction = TransactionBuilder::build_v0_transaction(payer, recent_blockhash, instructions, lookup_tables).map_err(|err| format!("Solana transaction error: {err}"))?;
    let bytes = transaction.serialize().map_err(|err| format!("Solana transaction error: {err}"))?;
    Ok(encode_base64(&bytes))
}

pub fn instruction_from_primitive(instruction: SolanaInstruction) -> Result<Instruction, String> {
    let program_id = Pubkey::from_base58(&instruction.program_id).map_err(|err| format!("Invalid Solana address {}: {err}", instruction.program_id))?;
    let accounts = instruction
        .accounts
        .into_iter()
        .map(|account| {
            Ok(AccountMeta {
                pubkey: Pubkey::from_base58(&account.pubkey).map_err(|err| format!("Invalid Solana address {}: {err}", account.pubkey))?,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Instruction {
        program_id,
        accounts,
        data: decode_base64(&instruction.data).map_err(|err| err.to_string())?,
    })
}

pub fn instructions_from_primitives(instructions: Vec<SolanaInstruction>) -> Result<Vec<Instruction>, String> {
    instructions.into_iter().map(instruction_from_primitive).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "signer")]
    use crate::signer::testkit::{SINGLE_SIG_TX, mock_legacy_transaction};

    #[test]
    fn test_try_decode_blockhash() {
        assert!(try_decode_blockhash("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").is_some());
        assert!(try_decode_blockhash("invalid blockhash").is_none());
        assert!(try_decode_blockhash("1111111111111111111111111111111").is_none());
    }

    #[cfg(feature = "signer")]
    #[test]
    fn test_is_transaction_bytes() {
        let full_transaction = gem_encoding::decode_base64(SINGLE_SIG_TX).unwrap();
        let transaction = VersionedTransaction::deserialize_with_version(&full_transaction).unwrap();
        let mut v0_message = transaction.serialize_message().unwrap();
        let mut transaction_with_trailing_byte = full_transaction.clone();

        assert!(is_transaction_bytes(&full_transaction));
        assert!(is_transaction_bytes(&v0_message));
        assert!(is_transaction_bytes(&mock_legacy_transaction().serialize_message().unwrap()));

        transaction_with_trailing_byte.push(0);
        v0_message.push(0);
        assert!(!is_transaction_bytes(&transaction_with_trailing_byte));
        assert!(!is_transaction_bytes(&v0_message));
        assert!(!is_transaction_bytes(b"hello"));
    }
}
