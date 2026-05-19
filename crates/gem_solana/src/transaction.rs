use gem_encoding::{decode_base64, encode_base64};
use primitives::SolanaInstruction;
use solana_primitives::{AccountMeta, AddressLookupTableAccount, Instruction, Pubkey, TransactionBuilder, VersionedTransaction};

pub fn try_decode_transaction(transaction_base64: &str) -> Option<VersionedTransaction> {
    let data = decode_base64(transaction_base64).ok()?;
    VersionedTransaction::deserialize_with_version(&data).ok()
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

    #[test]
    fn test_try_decode_blockhash() {
        assert!(try_decode_blockhash("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").is_some());
        assert!(try_decode_blockhash("invalid blockhash").is_none());
        assert!(try_decode_blockhash("1111111111111111111111111111111").is_none());
    }
}
