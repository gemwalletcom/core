use std::collections::{HashMap, HashSet};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use primitives::SolanaInstruction;
use solana_primitives::{CompiledInstruction, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes, SolanaError, VersionedMessageV0, VersionedTransaction};
use std::str::FromStr;

const LOOKUP_TABLE_META_SIZE: usize = 56;
const PUBKEY_SIZE: usize = 32;

pub struct ParsedInstruction {
    pub program_id: Pubkey,
    pub accounts: Vec<(Pubkey, bool, bool)>,
    pub data: Vec<u8>,
}

pub struct LookupTable {
    pub key: Pubkey,
    pub addresses: Vec<Pubkey>,
}

pub fn get_unit_limit(instructions: &[SolanaInstruction]) -> Option<u32> {
    instructions.iter().find_map(|instruction| {
        if instruction.program_id != crate::COMPUTE_BUDGET_PROGRAM_ID {
            return None;
        }
        let data = hex::decode(&instruction.data).ok()?;
        if data.len() == 5 && data[0] == crate::COMPUTE_UNIT_LIMIT_DISCRIMINANT {
            let units = u32::from_le_bytes(data[1..5].try_into().ok()?);
            Some((units as f64 * 1.2) as u32)
        } else {
            None
        }
    })
}

pub fn ensure_compute_unit_price(instructions: &mut Vec<SolanaInstruction>) {
    let has_price = instructions.iter().any(|instruction| {
        instruction.program_id == crate::COMPUTE_BUDGET_PROGRAM_ID && {
            let data = hex::decode(&instruction.data).unwrap_or_default();
            data.len() == 9 && data[0] == crate::COMPUTE_UNIT_PRICE_DISCRIMINANT
        }
    });
    if !has_price {
        let mut data = vec![crate::COMPUTE_UNIT_PRICE_DISCRIMINANT];
        data.extend_from_slice(&0u64.to_le_bytes());
        instructions.insert(
            0,
            SolanaInstruction {
                program_id: crate::COMPUTE_BUDGET_PROGRAM_ID.to_string(),
                accounts: vec![],
                data: hex::encode(data),
            },
        );
    }
}

pub fn parse_instructions(instructions: &[SolanaInstruction]) -> Result<Vec<ParsedInstruction>, SolanaError> {
    instructions
        .iter()
        .map(|instruction| {
            let accounts = instruction
                .accounts
                .iter()
                .map(|key| Ok((Pubkey::from_str(&key.pubkey)?, key.is_signer, key.is_writable)))
                .collect::<Result<_, SolanaError>>()?;
            Ok(ParsedInstruction {
                program_id: Pubkey::from_str(&instruction.program_id)?,
                accounts,
                data: hex::decode(&instruction.data).map_err(|_| SolanaError::InvalidInstructionData)?,
            })
        })
        .collect()
}

pub fn parse_lookup_table(key: &str, data: &[u8]) -> Result<LookupTable, SolanaError> {
    if data.len() < LOOKUP_TABLE_META_SIZE {
        return Err(SolanaError::InvalidMessage);
    }
    let addresses = data[LOOKUP_TABLE_META_SIZE..]
        .chunks_exact(PUBKEY_SIZE)
        .map(|chunk| Ok(Pubkey::new(chunk.try_into().map_err(|_| SolanaError::InvalidMessage)?)))
        .collect::<Result<_, SolanaError>>()?;
    Ok(LookupTable {
        key: Pubkey::from_str(key)?,
        addresses,
    })
}

pub fn build_v0_transaction(fee_payer: Pubkey, instructions: &[ParsedInstruction], lookup_tables: &[LookupTable], recent_blockhash: [u8; 32]) -> Result<String, SolanaError> {
    let lookup_map: HashMap<Pubkey, (usize, u8)> = lookup_tables
        .iter()
        .enumerate()
        .rev()
        .flat_map(|(ti, table)| {
            table
                .addresses
                .iter()
                .enumerate()
                .filter(|&(i, _)| i <= u8::MAX as usize)
                .map(move |(i, addr)| (*addr, (ti, i as u8)))
        })
        .collect();

    let program_ids: HashSet<Pubkey> = instructions.iter().map(|ix| ix.program_id).collect();

    let mut flags: HashMap<Pubkey, (bool, bool)> = HashMap::new();
    let mut order: Vec<Pubkey> = Vec::new();
    let mut merge = |pk: Pubkey, signer: bool, writable: bool| {
        flags
            .entry(pk)
            .and_modify(|(s, w)| {
                *s |= signer;
                *w |= writable;
            })
            .or_insert_with(|| {
                order.push(pk);
                (signer, writable)
            });
    };
    merge(fee_payer, true, true);
    for ix in instructions {
        merge(ix.program_id, false, false);
        for &(pk, signer, writable) in &ix.accounts {
            merge(pk, signer, writable);
        }
    }

    let mut static_keys: [Vec<Pubkey>; 4] = Default::default();
    let mut lookup_writable: Vec<Vec<(Pubkey, u8)>> = vec![vec![]; lookup_tables.len()];
    let mut lookup_readonly: Vec<Vec<(Pubkey, u8)>> = vec![vec![]; lookup_tables.len()];

    for &pk in &order {
        let (signer, writable) = flags[&pk];
        if signer || program_ids.contains(&pk) || !lookup_map.contains_key(&pk) {
            let bucket = match (signer, writable) {
                (true, true) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (false, false) => 3,
            };
            static_keys[bucket].push(pk);
        } else {
            let &(ti, ei) = lookup_map.get(&pk).ok_or(SolanaError::InvalidMessage)?;
            if writable {
                lookup_writable[ti].push((pk, ei));
            } else {
                lookup_readonly[ti].push((pk, ei));
            }
        }
    }

    let account_keys: Vec<Pubkey> = static_keys.iter().flat_map(|v| v.iter().copied()).collect();

    let header = MessageHeader {
        num_required_signatures: (static_keys[0].len() + static_keys[1].len()) as u8,
        num_readonly_signed_accounts: static_keys[1].len() as u8,
        num_readonly_unsigned_accounts: static_keys[3].len() as u8,
    };

    let virtual_index_map: HashMap<Pubkey, u8> = lookup_writable
        .iter()
        .flat_map(|v| v.iter())
        .chain(lookup_readonly.iter().flat_map(|v| v.iter()))
        .enumerate()
        .map(|(i, &(pk, _))| (pk, (account_keys.len() + i) as u8))
        .collect();

    let address_table_lookups: Vec<MessageAddressTableLookup> = lookup_tables
        .iter()
        .enumerate()
        .filter_map(|(i, table)| {
            let w: Vec<u8> = lookup_writable[i].iter().map(|&(_, ei)| ei).collect();
            let r: Vec<u8> = lookup_readonly[i].iter().map(|&(_, ei)| ei).collect();
            if w.is_empty() && r.is_empty() {
                return None;
            }
            Some(MessageAddressTableLookup::new(table.key, w, r))
        })
        .collect();

    let static_map: HashMap<Pubkey, u8> = account_keys.iter().enumerate().map(|(i, &pk)| (pk, i as u8)).collect();

    let compiled = instructions
        .iter()
        .map(|ix| {
            let program_id_index = *static_map.get(&ix.program_id).ok_or(SolanaError::InvalidMessage)?;
            let accounts = ix
                .accounts
                .iter()
                .map(|&(pk, _, _)| static_map.get(&pk).or_else(|| virtual_index_map.get(&pk)).copied().ok_or(SolanaError::InvalidMessage))
                .collect::<Result<_, _>>()?;
            Ok(CompiledInstruction {
                program_id_index,
                accounts,
                data: ix.data.clone(),
            })
        })
        .collect::<Result<Vec<_>, SolanaError>>()?;

    let transaction = VersionedTransaction::V0 {
        signatures: vec![SignatureBytes::default()],
        message: VersionedMessageV0 {
            header,
            account_keys,
            recent_blockhash,
            instructions: compiled,
            address_table_lookups,
        },
    };

    let bytes = transaction.serialize()?;
    Ok(STANDARD.encode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::SolanaAccountMeta;

    fn make_instruction(program_id: &str, keys: Vec<(&str, bool, bool)>, data: &str) -> SolanaInstruction {
        SolanaInstruction {
            program_id: program_id.to_string(),
            accounts: keys
                .into_iter()
                .map(|(pk, is_signer, is_writable)| SolanaAccountMeta {
                    pubkey: pk.to_string(),
                    is_signer,
                    is_writable,
                })
                .collect(),
            data: data.to_string(),
        }
    }

    fn build_test_tx(fee_payer: &str, raw: &[SolanaInstruction], tables: &[LookupTable]) -> Result<String, SolanaError> {
        let parsed = parse_instructions(raw)?;
        build_v0_transaction(Pubkey::from_str(fee_payer)?, &parsed, tables, [1u8; 32])
    }

    #[test]
    fn test_build_v0_no_lookup_tables() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let instructions = vec![make_instruction(
            "11111111111111111111111111111111",
            vec![(fee_payer, true, true), ("A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC", false, true)],
            "0200000040420f0000000000",
        )];

        let b64 = build_test_tx(fee_payer, &instructions, &[]).unwrap();
        let bytes = STANDARD.decode(&b64).unwrap();
        assert!(bytes.len() > 65);
        assert_eq!(bytes[65], 0x80);
    }

    #[test]
    fn test_build_v0_with_lookup_tables() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let account_in_table = Pubkey::new([42u8; 32]);
        let instructions = vec![make_instruction(
            "11111111111111111111111111111111",
            vec![(fee_payer, true, true), (&account_in_table.to_base58(), false, true)],
            "0200000040420f0000000000",
        )];
        let tables = vec![LookupTable {
            key: Pubkey::from_str("BZcyEKqjBNG5bEY6i5ev6PfPTgDSB9LwovJE1hJfJoHF").unwrap(),
            addresses: vec![account_in_table, Pubkey::new([99u8; 32])],
        }];

        let b64 = build_test_tx(fee_payer, &instructions, &tables).unwrap();
        let bytes = STANDARD.decode(&b64).unwrap();
        assert!(bytes.len() > 65);
        assert_eq!(bytes[65], 0x80);
    }

    #[test]
    fn test_get_unit_limit() {
        let with_limit = vec![
            make_instruction(crate::COMPUTE_BUDGET_PROGRAM_ID, vec![], "0259340000"),
            make_instruction("11111111111111111111111111111111", vec![], ""),
        ];
        assert_eq!(get_unit_limit(&with_limit), Some(16081));

        let without_limit = vec![make_instruction("11111111111111111111111111111111", vec![], "")];
        assert_eq!(get_unit_limit(&without_limit), None);
    }

    #[test]
    fn test_ensure_compute_unit_price() {
        let mut missing = vec![make_instruction(crate::COMPUTE_BUDGET_PROGRAM_ID, vec![], "0259340000")];
        ensure_compute_unit_price(&mut missing);
        assert_eq!(missing.len(), 2);
        assert_eq!(missing[0].data, "030000000000000000");

        let mut present = vec![
            make_instruction(crate::COMPUTE_BUDGET_PROGRAM_ID, vec![], "03e803000000000000"),
            make_instruction(crate::COMPUTE_BUDGET_PROGRAM_ID, vec![], "0259340000"),
        ];
        ensure_compute_unit_price(&mut present);
        assert_eq!(present.len(), 2);
        assert_eq!(present[0].data, "03e803000000000000");
    }
}
