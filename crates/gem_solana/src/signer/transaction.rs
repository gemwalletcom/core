use gem_encoding::encode_base64;
use num_traits::ToPrimitive;
use primitives::{SignerError, SignerInput, TransactionFee};
use solana_primitives::{
    CompiledInstruction, Instruction, Message, MessageHeader, Pubkey, SignatureBytes, Transaction,
    instructions::compute_budget::{set_compute_unit_limit, set_compute_unit_price},
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct AccountFlags {
    is_signer: bool,
    is_writable: bool,
}

pub(crate) fn compute_budget_instructions(fee: &TransactionFee) -> Result<Vec<Instruction>, SignerError> {
    let unit_price = fee.unit_price_u64()?;
    let gas_limit = fee.gas_limit.to_u32().ok_or_else(|| SignerError::invalid_input("invalid gas limit"))?;
    let mut instructions = Vec::new();
    if unit_price > 0 {
        instructions.push(set_compute_unit_price(unit_price));
    }
    if gas_limit > 0 {
        instructions.push(set_compute_unit_limit(gas_limit));
    }
    Ok(instructions)
}

pub(crate) fn sign_single_signer_instructions(input: &SignerInput, private_key: &[u8], fee_payer: Pubkey, instructions: Vec<Instruction>) -> Result<String, SignerError> {
    let mut transaction = build_legacy_transaction(fee_payer, block_hash(input)?, instructions)?;
    if transaction.num_required_signatures() != 1 {
        return Err(SignerError::invalid_input("Solana transaction requires more than one signer"));
    }
    transaction.sign(&[private_key]).map_err(|e| SignerError::signing_error(format!("sign: {e}")))?;
    let bytes = transaction
        .serialize_legacy()
        .map_err(|e| SignerError::signing_error(format!("serialize transaction: {e}")))?;
    Ok(encode_base64(&bytes))
}

fn build_legacy_transaction(fee_payer: Pubkey, recent_blockhash: [u8; 32], instructions: Vec<Instruction>) -> Result<Transaction, SignerError> {
    let mut flags = HashMap::new();
    let mut account_order = Vec::new();
    let mut program_order = Vec::new();

    merge_account(&mut flags, &mut account_order, fee_payer, true, true);
    for instruction in &instructions {
        for account in &instruction.accounts {
            merge_account(&mut flags, &mut account_order, account.pubkey, account.is_signer, account.is_writable);
        }
        merge_account(&mut flags, &mut program_order, instruction.program_id, false, false);
    }

    let mut buckets: [Vec<Pubkey>; 4] = Default::default();
    for pubkey in account_order.iter().chain(program_order.iter()) {
        let flags = flags.get(pubkey).ok_or_else(|| SignerError::invalid_input("missing Solana account flags"))?;
        let bucket = match (flags.is_signer, flags.is_writable) {
            (true, true) => 0,
            (true, false) => 1,
            (false, true) => 2,
            (false, false) => 3,
        };
        buckets[bucket].push(*pubkey);
    }

    let num_required_signatures = buckets[0].len() + buckets[1].len();
    let account_keys = account_keys(fee_payer, &buckets);
    if account_keys.len() > u8::MAX as usize || num_required_signatures > u8::MAX as usize {
        return Err(SignerError::invalid_input("Solana transaction has too many account keys"));
    }

    let key_to_index = account_keys.iter().enumerate().map(|(index, pubkey)| (*pubkey, index as u8)).collect::<HashMap<_, _>>();
    let compiled_instructions = instructions
        .iter()
        .map(|instruction| {
            let program_id_index = account_index(&key_to_index, instruction.program_id)?;
            let accounts = instruction
                .accounts
                .iter()
                .map(|account| account_index(&key_to_index, account.pubkey))
                .collect::<Result<Vec<_>, SignerError>>()?;
            Ok(CompiledInstruction {
                program_id_index,
                accounts,
                data: instruction.data.clone(),
            })
        })
        .collect::<Result<Vec<_>, SignerError>>()?;

    let header = MessageHeader {
        num_required_signatures: num_required_signatures as u8,
        num_readonly_signed_accounts: buckets[1].len() as u8,
        num_readonly_unsigned_accounts: buckets[3].len() as u8,
    };
    Ok(Transaction {
        signatures: vec![SignatureBytes::new([0u8; 64]); num_required_signatures],
        message: Message::new(header, account_keys, recent_blockhash, compiled_instructions),
    })
}

fn merge_account(flags: &mut HashMap<Pubkey, AccountFlags>, order: &mut Vec<Pubkey>, pubkey: Pubkey, is_signer: bool, is_writable: bool) {
    flags
        .entry(pubkey)
        .and_modify(|flags| {
            flags.is_signer |= is_signer;
            flags.is_writable |= is_writable;
        })
        .or_insert_with(|| {
            order.push(pubkey);
            AccountFlags { is_signer, is_writable }
        });
}

fn account_keys(fee_payer: Pubkey, buckets: &[Vec<Pubkey>; 4]) -> Vec<Pubkey> {
    let mut account_keys = Vec::with_capacity(buckets.iter().map(Vec::len).sum());
    account_keys.push(fee_payer);
    account_keys.extend(buckets[0].iter().copied().filter(|pubkey| *pubkey != fee_payer));
    for bucket in &buckets[1..] {
        account_keys.extend(bucket.iter().copied());
    }
    account_keys
}

fn account_index(key_to_index: &HashMap<Pubkey, u8>, pubkey: Pubkey) -> Result<u8, SignerError> {
    key_to_index.get(&pubkey).copied().ok_or_else(|| SignerError::invalid_input("missing Solana account key"))
}

fn block_hash(input: &SignerInput) -> Result<[u8; 32], SignerError> {
    let block_hash = input.metadata.get_block_hash()?;
    let bytes = bs58::decode(&block_hash).into_vec().map_err(|_| SignerError::invalid_input("invalid Solana block hash"))?;
    bytes.try_into().map_err(|_| SignerError::invalid_input("Solana block hash must be 32 bytes"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode_transaction, signer::testkit::SINGLE_SIG_TX};
    use solana_primitives::AccountMeta;

    fn pubkey(value: u8) -> Pubkey {
        Pubkey::new([value; 32])
    }

    #[test]
    fn test_decode_transaction_compute_unit_limit() {
        let transaction = decode_transaction(SINGLE_SIG_TX).unwrap();

        assert_eq!(transaction.get_compute_unit_limit(), Some(1_400_000));
    }

    #[test]
    fn test_build_legacy_transaction_preserves_account_order_by_bucket() {
        let fee_payer = pubkey(1);
        let writable = pubkey(2);
        let readonly_first = pubkey(3);
        let readonly_second = pubkey(4);
        let program_first = pubkey(5);
        let program_second = pubkey(6);
        let instructions = vec![
            Instruction {
                program_id: program_first,
                accounts: vec![
                    AccountMeta {
                        pubkey: fee_payer,
                        is_signer: true,
                        is_writable: false,
                    },
                    AccountMeta {
                        pubkey: readonly_first,
                        is_signer: false,
                        is_writable: false,
                    },
                    AccountMeta {
                        pubkey: writable,
                        is_signer: false,
                        is_writable: true,
                    },
                ],
                data: vec![1],
            },
            Instruction {
                program_id: program_second,
                accounts: vec![AccountMeta {
                    pubkey: readonly_second,
                    is_signer: false,
                    is_writable: false,
                }],
                data: vec![2],
            },
        ];

        let transaction = build_legacy_transaction(fee_payer, [0; 32], instructions).unwrap();

        assert_eq!(
            transaction.account_keys(),
            &[fee_payer, writable, readonly_first, readonly_second, program_first, program_second]
        );
        assert_eq!(transaction.num_required_signatures(), 1);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 4);
    }

    #[test]
    fn test_build_legacy_transaction_upgrades_duplicate_account_flags() {
        let fee_payer = pubkey(1);
        let upgraded = pubkey(2);
        let program = pubkey(3);
        let instructions = vec![
            Instruction {
                program_id: program,
                accounts: vec![AccountMeta {
                    pubkey: upgraded,
                    is_signer: false,
                    is_writable: false,
                }],
                data: vec![1],
            },
            Instruction {
                program_id: program,
                accounts: vec![AccountMeta {
                    pubkey: upgraded,
                    is_signer: false,
                    is_writable: true,
                }],
                data: vec![2],
            },
        ];

        let transaction = build_legacy_transaction(fee_payer, [0; 32], instructions).unwrap();

        assert_eq!(transaction.account_keys(), &[fee_payer, upgraded, program]);
        assert_eq!(transaction.num_readonly_unsigned_accounts(), 1);
    }
}
