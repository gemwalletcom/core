use crate::{Quote, RpcProvider, SwapperError, SwapperQuoteData, client_factory::create_client_with_chain, mayan::constants::MAYAN_CPI_PROXY_PROGRAM_ID};
use futures::try_join;
use gem_encoding::decode_base64;
use gem_solana::{ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SYSTEM_PROGRAM_ID, SolanaAddress, SolanaClient, encode_v0_transaction, instruction_from_primitive};
use primitives::{Chain, SolanaInstruction};
use solana_primitives::{AccountMeta, Instruction, Pubkey, compute_budget};
use std::{fmt::Display, sync::Arc};

#[derive(Debug)]
pub(in crate::mayan::tx_builder) struct SolanaTransaction {
    instructions: Vec<Instruction>,
    lookup_table_addresses: Vec<String>,
}

impl SolanaTransaction {
    pub(in crate::mayan::tx_builder) fn new(instructions: Vec<Instruction>, lookup_table_addresses: Vec<String>) -> Self {
        Self {
            instructions,
            lookup_table_addresses,
        }
    }
}

pub(in crate::mayan::tx_builder) async fn build_quote_data(
    quote: &Quote,
    transaction: SolanaTransaction,
    rpc_provider: Arc<dyn RpcProvider>,
) -> Result<SwapperQuoteData, SwapperError> {
    let rpc_client = SolanaClient::new(create_client_with_chain(rpc_provider, Chain::Solana));
    let lookup_tables = async { rpc_client.get_address_lookup_tables(transaction.lookup_table_addresses).await.map_err(solana_error) };
    let blockhash = async { rpc_client.get_latest_blockhash().await.map(|response| response.value.blockhash).map_err(SwapperError::from) };
    let (lookup_tables, blockhash) = try_join!(lookup_tables, blockhash)?;
    let fee_payer = SolanaAddress::parse(&quote.request.wallet_address).map_err(solana_error)?.into();
    let data = encode_v0_transaction(fee_payer, &blockhash, &transaction.instructions, &lookup_tables).map_err(solana_error)?;
    let gas_limit = compute_budget::get_compute_unit_limit(&transaction.instructions).map(|limit| limit.to_string());

    Ok(SwapperQuoteData::new_contract(String::new(), "0".to_string(), data, None, gas_limit))
}

pub(in crate::mayan::tx_builder) fn setup_instructions(instructions: Vec<SolanaInstruction>, payer: &Pubkey) -> Result<Vec<Instruction>, SwapperError> {
    instructions
        .into_iter()
        .map(|instruction| {
            override_setup_payer(instruction, payer)
                .and_then(|instruction| instruction_from_primitive(instruction).map_err(solana_error))
                .and_then(wrap_instruction_in_cpi_proxy)
        })
        .collect()
}

fn override_setup_payer(mut instruction: SolanaInstruction, payer: &Pubkey) -> Result<SolanaInstruction, SwapperError> {
    if instruction.accounts.is_empty() {
        return Ok(instruction);
    }
    let data = decode_base64(&instruction.data).map_err(solana_error)?;
    let should_override = match instruction.program_id.as_str() {
        SYSTEM_PROGRAM_ID => data.starts_with(&[0, 0, 0, 0]),
        ASSOCIATED_TOKEN_ACCOUNT_PROGRAM => data.is_empty() || data.as_slice() == [1],
        _ => false,
    };
    if should_override {
        instruction.accounts[0].pubkey = payer.to_string();
    }
    Ok(instruction)
}

pub(in crate::mayan::tx_builder) fn wrap_instruction_in_cpi_proxy(instruction: Instruction) -> Result<Instruction, SwapperError> {
    let mut accounts = Vec::with_capacity(instruction.accounts.len() + 1);
    accounts.push(AccountMeta::new_readonly(instruction.program_id));
    accounts.extend(instruction.accounts);
    Ok(Instruction {
        program_id: SolanaAddress::parse(MAYAN_CPI_PROXY_PROGRAM_ID).map_err(solana_error)?.into(),
        accounts,
        data: instruction.data,
    })
}

pub(in crate::mayan::tx_builder) fn solana_error(err: impl Display) -> SwapperError {
    SwapperError::transaction_error(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_instruction_in_cpi_proxy_uses_deployed_program_id() {
        let program_id = Pubkey::new([1; 32]);
        let account = Pubkey::new([2; 32]);
        let instruction = Instruction {
            program_id,
            accounts: vec![AccountMeta::new_readonly(account)],
            data: vec![1, 2, 3],
        };

        let wrapped = wrap_instruction_in_cpi_proxy(instruction).unwrap();

        assert_eq!(wrapped.program_id.to_string(), MAYAN_CPI_PROXY_PROGRAM_ID);
        assert_eq!(wrapped.accounts.len(), 2);
        assert_eq!(wrapped.accounts[0].pubkey, program_id);
        assert!(!wrapped.accounts[0].is_signer);
        assert!(!wrapped.accounts[0].is_writable);
        assert_eq!(wrapped.accounts[1].pubkey, account);
        assert!(!wrapped.accounts[1].is_signer);
        assert!(!wrapped.accounts[1].is_writable);
        assert_eq!(wrapped.data, vec![1, 2, 3]);
    }
}
