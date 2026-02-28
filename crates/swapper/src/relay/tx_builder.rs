use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::hex;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, COMPUTE_BUDGET_PROGRAM_ID, COMPUTE_UNIT_LIMIT_DATA, COMPUTE_UNIT_PRICE_DATA, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM, WSOL_TOKEN_ADDRESS,
    jsonrpc::SolanaRpc,
    models::{
        LatestBlockhash,
        rpc::{AccountData, ValueResult},
    },
    token_account::get_token_account,
};
use primitives::{Chain, SolanaAccountMeta, SolanaInstruction};
use solana_primitives::{
    CompiledInstruction, MessageAddressTableLookup, MessageHeader, Pubkey, SignatureBytes, VersionedMessageV0, VersionedTransaction, instructions::system::SystemInstruction,
    instructions::token::TokenInstruction,
};

use crate::{SwapperError, alien::RpcProvider, client_factory::create_client_with_chain};

const LOOKUP_TABLE_META_SIZE: usize = 56;
const PUBKEY_SIZE: usize = 32;

struct ParsedInstruction {
    program_id: Pubkey,
    accounts: Vec<(Pubkey, bool, bool)>,
    data: Vec<u8>,
}

struct LookupTable {
    key: Pubkey,
    addresses: Vec<Pubkey>,
}

pub async fn build_solana_tx(
    fee_payer: &str,
    instructions: &[SolanaInstruction],
    lookup_table_addresses: &[String],
    provider: Arc<dyn RpcProvider>,
    wrap_sol_amount: Option<u64>,
) -> Result<String, SwapperError> {
    let fee_payer_pk = Pubkey::from_str(fee_payer)?;
    let mut instructions = ensure_compute_budget_instructions(instructions);
    if let Some(amount) = wrap_sol_amount {
        ensure_wrap_wsol(&mut instructions, fee_payer, amount);
    }
    let parsed = parse_instructions(&instructions)?;

    let (recent_blockhash, lookup_tables) = futures::try_join!(fetch_recent_blockhash(&provider), fetch_lookup_tables(&provider, lookup_table_addresses))?;

    build_v0_transaction(fee_payer_pk, &parsed, &lookup_tables, recent_blockhash)
}

async fn fetch_recent_blockhash(provider: &Arc<dyn RpcProvider>) -> Result<[u8; 32], SwapperError> {
    let client = create_client_with_chain(provider.clone(), Chain::Solana);
    let response: LatestBlockhash = client
        .request(SolanaRpc::GetLatestBlockhash)
        .await
        .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;
    let bytes = bs58::decode(&response.value.blockhash).into_vec().map_err(|_| SwapperError::InvalidRoute)?;
    bytes.try_into().map_err(|_| SwapperError::InvalidRoute)
}

async fn fetch_lookup_tables(provider: &Arc<dyn RpcProvider>, addresses: &[String]) -> Result<Vec<LookupTable>, SwapperError> {
    if addresses.is_empty() {
        return Ok(vec![]);
    }
    let client = create_client_with_chain(provider.clone(), Chain::Solana);
    let response: ValueResult<Vec<Option<AccountData>>> = client
        .request(SolanaRpc::GetMultipleAccounts(addresses.to_vec()))
        .await
        .map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;

    addresses
        .iter()
        .zip(response.value.iter())
        .map(|(address, account)| {
            let account = account.as_ref().ok_or(SwapperError::InvalidRoute)?;
            let data = STANDARD
                .decode(account.data.first().ok_or(SwapperError::InvalidRoute)?)
                .map_err(|_| SwapperError::InvalidRoute)?;
            if data.len() < LOOKUP_TABLE_META_SIZE {
                return Err(SwapperError::InvalidRoute);
            }
            let addresses = data[LOOKUP_TABLE_META_SIZE..]
                .chunks_exact(PUBKEY_SIZE)
                .map(|chunk| Ok(Pubkey::new(chunk.try_into().map_err(|_| SwapperError::InvalidRoute)?)))
                .collect::<Result<_, SwapperError>>()?;
            Ok(LookupTable {
                key: Pubkey::from_str(address)?,
                addresses,
            })
        })
        .collect()
}

fn ensure_compute_budget_instructions(instructions: &[SolanaInstruction]) -> Vec<SolanaInstruction> {
    let budget_ix = |data: &str| SolanaInstruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID.to_string(),
        accounts: vec![],
        data: data.to_string(),
    };
    let has_limit = instructions.iter().any(|i| i.program_id == COMPUTE_BUDGET_PROGRAM_ID && i.data.starts_with("02"));
    let has_price = instructions.iter().any(|i| i.program_id == COMPUTE_BUDGET_PROGRAM_ID && i.data.starts_with("03"));
    let mut result = Vec::new();
    if !has_limit {
        result.push(budget_ix(COMPUTE_UNIT_LIMIT_DATA));
    }
    if !has_price {
        result.push(budget_ix(COMPUTE_UNIT_PRICE_DATA));
    }
    result.extend(instructions.iter().cloned());
    result
}

fn ensure_wrap_wsol(instructions: &mut Vec<SolanaInstruction>, fee_payer: &str, amount: u64) {
    let wsol_ata = get_token_account(fee_payer, WSOL_TOKEN_ADDRESS, TOKEN_PROGRAM);
    let transfer_prefix = hex::encode(&SystemInstruction::Transfer { lamports: 0 }.serialize()[..4]);

    let has_wsol_transfer = instructions
        .iter()
        .any(|ix| ix.program_id == SYSTEM_PROGRAM_ID && ix.data.starts_with(&transfer_prefix) && ix.accounts.len() >= 2 && ix.accounts[1].pubkey == wsol_ata);
    if has_wsol_transfer {
        return;
    }

    let transfer_data = SystemInstruction::Transfer { lamports: amount }.serialize();
    let sync_native_data = TokenInstruction::SyncNative.serialize();

    let wrap_instructions = vec![
        SolanaInstruction {
            program_id: SYSTEM_PROGRAM_ID.to_string(),
            accounts: vec![
                SolanaAccountMeta {
                    pubkey: fee_payer.to_string(),
                    is_signer: true,
                    is_writable: true,
                },
                SolanaAccountMeta {
                    pubkey: wsol_ata.clone(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: hex::encode(transfer_data),
        },
        SolanaInstruction {
            program_id: TOKEN_PROGRAM.to_string(),
            accounts: vec![SolanaAccountMeta {
                pubkey: wsol_ata,
                is_signer: false,
                is_writable: true,
            }],
            data: hex::encode(sync_native_data),
        },
    ];

    let insert_pos = instructions
        .iter()
        .rposition(|i| i.program_id == ASSOCIATED_TOKEN_ACCOUNT_PROGRAM)
        .map(|i| i + 1)
        .unwrap_or_else(|| instructions.iter().position(|i| i.program_id != COMPUTE_BUDGET_PROGRAM_ID).unwrap_or(instructions.len()));

    for (offset, ix) in wrap_instructions.into_iter().enumerate() {
        instructions.insert(insert_pos + offset, ix);
    }
}

fn parse_instructions(instructions: &[SolanaInstruction]) -> Result<Vec<ParsedInstruction>, SwapperError> {
    instructions
        .iter()
        .map(|instruction| {
            let accounts = instruction
                .accounts
                .iter()
                .map(|key| Ok((Pubkey::from_str(&key.pubkey)?, key.is_signer, key.is_writable)))
                .collect::<Result<_, SwapperError>>()?;
            Ok(ParsedInstruction {
                program_id: Pubkey::from_str(&instruction.program_id)?,
                accounts,
                data: hex::decode(&instruction.data).map_err(|_| SwapperError::InvalidRoute)?,
            })
        })
        .collect()
}

fn build_v0_transaction(fee_payer: Pubkey, instructions: &[ParsedInstruction], lookup_tables: &[LookupTable], recent_blockhash: [u8; 32]) -> Result<String, SwapperError> {
    // Pubkey → (table_index, index_in_table). rev() gives first-table-wins via last-write-wins in collect.
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

    // Collect unique accounts with merged (signer, writable) flags, preserving insertion order
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

    // Categorize: [writable_signers, readonly_signers, writable_unsigned, readonly_unsigned]
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
            let &(ti, ei) = lookup_map.get(&pk).ok_or(SwapperError::InvalidRoute)?;
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

    // Virtual indices: writable lookups across all tables first, then readonly
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
            let program_id_index = *static_map.get(&ix.program_id).ok_or(SwapperError::InvalidRoute)?;
            let accounts = ix
                .accounts
                .iter()
                .map(|&(pk, _, _)| static_map.get(&pk).or_else(|| virtual_index_map.get(&pk)).copied().ok_or(SwapperError::InvalidRoute))
                .collect::<Result<_, _>>()?;
            Ok(CompiledInstruction {
                program_id_index,
                accounts,
                data: ix.data.clone(),
            })
        })
        .collect::<Result<Vec<_>, SwapperError>>()?;

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

    let bytes = transaction.serialize().map_err(|e| SwapperError::ComputeQuoteError(e.to_string()))?;
    Ok(STANDARD.encode(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn build_test_tx(fee_payer: &str, raw: &[SolanaInstruction], tables: &[LookupTable]) -> Result<String, SwapperError> {
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
    fn test_ensure_compute_budget_prepended() {
        let result = ensure_compute_budget_instructions(&[make_instruction("11111111111111111111111111111111", vec![], "00")]);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].program_id, COMPUTE_BUDGET_PROGRAM_ID);
        assert_eq!(result[1].program_id, COMPUTE_BUDGET_PROGRAM_ID);
    }

    #[test]
    fn test_ensure_compute_budget_adds_missing_price() {
        let instructions = vec![
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "0200000000"),
            make_instruction("11111111111111111111111111111111", vec![], "00"),
        ];
        let result = ensure_compute_budget_instructions(&instructions);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].data, COMPUTE_UNIT_PRICE_DATA);
    }

    #[test]
    fn test_ensure_compute_budget_both_present() {
        let instructions = vec![
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "0200000000"),
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "030000000000000000"),
            make_instruction("11111111111111111111111111111111", vec![], "00"),
        ];
        assert_eq!(ensure_compute_budget_instructions(&instructions).len(), 3);
    }

    #[test]
    fn test_build_v0_includes_compute_budget() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let raw = ensure_compute_budget_instructions(&[make_instruction(
            "11111111111111111111111111111111",
            vec![(fee_payer, true, true), ("A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC", false, true)],
            "0200000040420f0000000000",
        )]);

        let b64 = build_test_tx(fee_payer, &raw, &[]).unwrap();
        let bytes = STANDARD.decode(&b64).unwrap();
        let tx = VersionedTransaction::deserialize_with_version(&bytes).unwrap();
        assert_eq!(tx.get_compute_unit_limit(), Some(0));

        let mut tx2 = VersionedTransaction::deserialize_with_version(&bytes).unwrap();
        assert!(tx2.set_compute_unit_price(1000).unwrap());
        assert!(tx2.set_compute_unit_limit(200_000).unwrap());
    }

    #[test]
    fn test_build_v0_invalid_program_id() {
        let raw = vec![SolanaInstruction {
            program_id: "invalid".to_string(),
            accounts: vec![],
            data: String::new(),
        }];
        assert!(build_test_tx(&Pubkey::new([1u8; 32]).to_base58(), &raw, &[]).is_err());
    }

    #[test]
    fn test_ensure_wrap_wsol_inserts_after_ata() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let mut instructions = vec![
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "0200000000"),
            make_instruction(ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, vec![], "00"),
            make_instruction("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH", vec![], "aabb"),
        ];
        ensure_wrap_wsol(&mut instructions, fee_payer, 200_000_000);
        assert_eq!(instructions.len(), 5);
        assert_eq!(instructions[2].program_id, SYSTEM_PROGRAM_ID);
        assert_eq!(instructions[3].program_id, TOKEN_PROGRAM);
    }

    #[test]
    fn test_ensure_wrap_wsol_skips_when_present() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let wsol_ata = get_token_account(fee_payer, WSOL_TOKEN_ADDRESS, TOKEN_PROGRAM);
        let transfer_data = hex::encode(SystemInstruction::Transfer { lamports: 500_000_000 }.serialize());
        let mut instructions = vec![
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "0200000000"),
            make_instruction(SYSTEM_PROGRAM_ID, vec![(fee_payer, true, true), (&wsol_ata, false, true)], &transfer_data),
            make_instruction("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH", vec![], "aabb"),
        ];
        ensure_wrap_wsol(&mut instructions, fee_payer, 200_000_000);
        assert_eq!(instructions.len(), 3);
    }

    #[test]
    fn test_ensure_wrap_wsol_no_ata() {
        let fee_payer = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy";
        let mut instructions = vec![
            make_instruction(COMPUTE_BUDGET_PROGRAM_ID, vec![], "0200000000"),
            make_instruction("DF1ow4tspfHX9JwWJsAb9epbkA8hmpSEAtxXy1V27QBH", vec![], "aabb"),
        ];
        ensure_wrap_wsol(&mut instructions, fee_payer, 50_000_000);
        assert_eq!(instructions.len(), 4);
        assert_eq!(instructions[1].program_id, SYSTEM_PROGRAM_ID);
        assert_eq!(instructions[2].program_id, TOKEN_PROGRAM);
    }
}
