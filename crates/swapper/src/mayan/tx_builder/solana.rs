use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    client_factory::create_client_with_chain,
    mayan::{constants::MAYAN_CPI_PROXY_PROGRAM_ID, model::SolanaClientSwap},
};
use futures::try_join;
use gem_encoding::decode_base64;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SYSTEM_PROGRAM_ID, SolanaAddress, SolanaClient, WSOL_TOKEN_ADDRESS, encode_v0_transaction, instruction_from_primitive,
    instructions_from_primitives,
};
use primitives::{Chain, SolanaInstruction};
use solana_primitives::associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id};
use solana_primitives::instructions::program_ids;
use solana_primitives::{AccountMeta, Instruction, Pubkey, compute_budget, system, token};
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

pub(in crate::mayan::tx_builder) struct SolanaLedgerDeposit<'a> {
    pub user: &'a Pubkey,
    pub relayer: &'a Pubkey,
    pub ledger: &'a Pubkey,
    pub ledger_account: &'a Pubkey,
    pub mint: &'a Pubkey,
    pub amount: u64,
    pub suggested_priority_fee: Option<u64>,
}

pub(in crate::mayan::tx_builder) fn append_ledger_deposit_instructions(instructions: &mut Vec<Instruction>, deposit: SolanaLedgerDeposit<'_>) -> Result<(), SwapperError> {
    if let Some(priority_fee) = deposit.suggested_priority_fee.filter(|&fee| fee > 0) {
        instructions.push(compute_budget::set_compute_unit_price(priority_fee));
    }

    instructions.push(wrap_instruction_in_cpi_proxy(create_associated_token_account_idempotent_with_address(
        deposit.relayer,
        deposit.ledger_account,
        deposit.ledger,
        deposit.mint,
        &program_ids::token_program(),
    ))?);

    let source_account = get_associated_token_address_with_program_id(deposit.user, deposit.mint, &program_ids::token_program());
    instructions.push(wrap_instruction_in_cpi_proxy(token::transfer(
        &source_account,
        deposit.ledger_account,
        deposit.user,
        deposit.amount,
    ))?);
    Ok(())
}

pub(in crate::mayan::tx_builder) fn append_client_swap_instructions(
    instructions: &mut Vec<Instruction>,
    swap: SolanaClientSwap,
    user: &Pubkey,
    relayer: &Pubkey,
    from_token_contract: &str,
    amount_in64: &str,
) -> Result<Vec<String>, SwapperError> {
    let setup = swap.setup_instructions.unwrap_or_default();
    let compute_budget = swap.compute_budget_instructions.unwrap_or_default();
    instructions.extend(instructions_from_primitives(compute_budget).map_err(solana_error)?);
    if from_token_contract == EVM_ZERO_ADDRESS && !setup_wraps_native_sol(&setup, user)? {
        instructions.extend(wrap_native_sol_instructions(user, amount_in64.parse::<u64>()?)?);
    }
    instructions.extend(setup_instructions(setup, relayer)?);
    instructions.push(instruction_from_primitive(swap.swap_instruction).map_err(solana_error)?);
    if let Some(cleanup_instruction) = swap.cleanup_instruction {
        instructions.push(wrap_instruction_in_cpi_proxy(instruction_from_primitive(cleanup_instruction).map_err(solana_error)?)?);
    }
    Ok(swap.address_lookup_table_addresses)
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

pub(in crate::mayan::tx_builder) fn setup_wraps_native_sol(instructions: &[SolanaInstruction], owner: &Pubkey) -> Result<bool, SwapperError> {
    let wrapped_account = wrapped_sol_account(owner)?;
    Ok(instructions.iter().any(|instruction| {
        instruction.program_id == SYSTEM_PROGRAM_ID
            && instruction.accounts.get(1).is_some_and(|account| account.pubkey == wrapped_account.to_string())
            && decode_base64(&instruction.data).is_ok_and(|data| data.starts_with(&[2, 0, 0, 0]))
    }))
}

pub(in crate::mayan::tx_builder) fn wrap_native_sol_instructions(owner: &Pubkey, amount: u64) -> Result<Vec<Instruction>, SwapperError> {
    let wrapped_mint = wrapped_sol_mint()?;
    let wrapped_account = get_associated_token_address_with_program_id(owner, &wrapped_mint, &program_ids::token_program());
    Ok(vec![
        wrap_instruction_in_cpi_proxy(create_associated_token_account_idempotent_with_address(
            owner,
            &wrapped_account,
            owner,
            &wrapped_mint,
            &program_ids::token_program(),
        ))?,
        wrap_instruction_in_cpi_proxy(system::transfer(owner, &wrapped_account, amount))?,
        wrap_instruction_in_cpi_proxy(token::sync_native(&wrapped_account))?,
    ])
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

fn wrapped_sol_account(owner: &Pubkey) -> Result<Pubkey, SwapperError> {
    Ok(get_associated_token_address_with_program_id(owner, &wrapped_sol_mint()?, &program_ids::token_program()))
}

fn wrapped_sol_mint() -> Result<Pubkey, SwapperError> {
    Ok(SolanaAddress::parse(WSOL_TOKEN_ADDRESS).map_err(solana_error)?.into())
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
