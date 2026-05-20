use super::{
    order::{create_init_instruction, create_order_hash},
    payload::{create_payload_writer_close_instruction, create_payload_writer_create_instruction},
    solana_error,
};
use crate::{
    Quote, SwapperError,
    fees::default_referral_address,
    mayan::{
        client::MayanClient,
        constants::{MAYAN_CPI_PROXY_PROGRAM_ID, MAYAN_LOOKUP_TABLE_SOLANA, MAYAN_PAYLOAD_WRITER_PROGRAM_ID, MAYAN_SWIFT_V2_PROGRAM_ID},
        model::{GetSwapSolanaParams, MayanSwiftQuote, SolanaClientSwap},
        tx_builder::{
            amount::value_to_query,
            hypercore::hypercore_custom_payload,
            route::{quote_destination_address, swift_destination_address},
            swift::{SwiftOrderFields, swift_input_contract as route_swift_input_contract, swift_random_key},
        },
    },
};
use gem_client::Client;
use gem_encoding::decode_base64;
use gem_evm::EVM_ZERO_ADDRESS;
use gem_solana::{ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SYSTEM_PROGRAM_ID, SolanaAddress, WSOL_TOKEN_ADDRESS, instruction_from_primitive, instructions_from_primitives};
use primitives::{Chain, SolanaInstruction};
use rand::RngExt;
use solana_primitives::associated_token::{create_associated_token_account_idempotent_with_address, get_associated_token_address_with_program_id};
use solana_primitives::instructions::program_ids;
use solana_primitives::{AccountMeta, Instruction, Pubkey, compute_budget, find_program_address, system, token};
use std::fmt::Debug;

#[derive(Debug)]
pub(super) struct SolanaSwapTransaction {
    pub(super) instructions: Vec<Instruction>,
    pub(super) lookup_table_addresses: Vec<String>,
}

struct SwiftBuildContext {
    trader: Pubkey,
    destination_address: String,
    custom_payload: Option<Vec<u8>>,
    referrer_address: Option<String>,
    random_key: [u8; 32],
    state: Pubkey,
    token_program: Pubkey,
    swift_input_contract: String,
    swift_input_mint: Pubkey,
    state_account: Pubkey,
    relayer: Pubkey,
    relayer_account: Pubkey,
    order_fields: SwiftOrderFields,
}

impl SwiftBuildContext {
    fn new(quote: &Quote, route: &MayanSwiftQuote) -> Result<Self, SwapperError> {
        let trader = SolanaAddress::parse(&quote.request.wallet_address).map_err(solana_error)?.into();
        let destination_address = swift_destination_address(quote, route).into_owned();
        let custom_payload = hypercore_custom_payload(route, quote_destination_address(quote))?;
        let referrer_address = Some(default_referral_address(Chain::Solana));
        let random_key = swift_random_key(route)?;
        let order_fields = SwiftOrderFields::new(route)?;
        let order_hash = create_order_hash(
            route,
            &quote.request.wallet_address,
            &destination_address,
            &random_key,
            custom_payload.as_deref(),
            &order_fields,
        )?;
        let destination_chain_bytes = order_fields.destination_chain_id.to_le_bytes();
        let swift_program = SolanaAddress::parse(MAYAN_SWIFT_V2_PROGRAM_ID).map_err(solana_error)?.into();
        let (state, _) = find_program_address(&swift_program, &[b"STATE_SOURCE", &order_hash, &destination_chain_bytes]).map_err(solana_error)?;
        let token_program = swift_token_program(route);
        let swift_input_contract = route_swift_input_contract(route)?.to_string();
        let swift_input_mint = if swift_input_contract == EVM_ZERO_ADDRESS {
            SolanaAddress::parse(WSOL_TOKEN_ADDRESS).map_err(solana_error)?.into()
        } else {
            SolanaAddress::parse(&swift_input_contract).map_err(solana_error)?.into()
        };
        let state_account = get_associated_token_address_with_program_id(&state, &swift_input_mint, &token_program);
        let relayer = trader;
        let relayer_account = get_associated_token_address_with_program_id(&relayer, &swift_input_mint, &token_program);

        Ok(Self {
            trader,
            destination_address,
            custom_payload,
            referrer_address,
            random_key,
            state,
            token_program,
            swift_input_contract,
            swift_input_mint,
            state_account,
            relayer,
            relayer_account,
            order_fields,
        })
    }
}

pub(super) async fn build<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanSwiftQuote) -> Result<SolanaSwapTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let context = SwiftBuildContext::new(quote, route)?;
    let mut instructions = Vec::new();
    let mut lookup_table_addresses = vec![MAYAN_LOOKUP_TABLE_SOLANA.to_string()];
    let custom_payload_account = create_custom_payload_account(&mut instructions, &context.relayer, context.custom_payload.as_deref())?;

    if route.from_token.contract.as_str() == context.swift_input_contract.as_str() {
        add_direct_swift_instructions(route, &context, &mut instructions)?;
    } else {
        lookup_table_addresses.extend(add_swap_instructions(client, quote, route, &context, &mut instructions).await?);
    }

    instructions.push(wrap_instruction_in_cpi_proxy(create_init_instruction(
        route,
        &context.state,
        &context.trader,
        &context.relayer,
        &context.state_account,
        &context.relayer_account,
        &context.swift_input_mint,
        &context.token_program,
        &context.destination_address,
        context.random_key,
        custom_payload_account.as_ref().map(|(account, _)| account),
        &context.order_fields,
    )?)?);

    if let Some((payload_account, nonce)) = custom_payload_account {
        instructions.push(wrap_instruction_in_cpi_proxy(create_payload_writer_close_instruction(
            &context.relayer,
            &payload_account,
            nonce,
        )?)?);
    }

    Ok(SolanaSwapTransaction {
        instructions,
        lookup_table_addresses,
    })
}

fn add_direct_swift_instructions(route: &MayanSwiftQuote, context: &SwiftBuildContext, instructions: &mut Vec<Instruction>) -> Result<(), SwapperError> {
    if let Some(priority_fee) = route.suggested_priority_fee.filter(|&fee| fee > 0) {
        instructions.push(wrap_instruction_in_cpi_proxy(compute_budget::set_compute_unit_price(priority_fee))?);
    }
    instructions.push(wrap_instruction_in_cpi_proxy(create_associated_token_account_idempotent_with_address(
        &context.relayer,
        &context.state_account,
        &context.state,
        &context.swift_input_mint,
        &context.token_program,
    ))?);

    let amount_in = route.effective_amount_in64.parse::<u64>()?;
    if context.swift_input_contract == EVM_ZERO_ADDRESS {
        instructions.push(wrap_instruction_in_cpi_proxy(system::transfer(&context.trader, &context.state_account, amount_in))?);
        instructions.push(wrap_instruction_in_cpi_proxy(token::sync_native(&context.state_account))?);
        return Ok(());
    }

    let source_account = get_associated_token_address_with_program_id(&context.trader, &context.swift_input_mint, &context.token_program);
    let mut transfer = token::transfer(&source_account, &context.state_account, &context.trader, amount_in);
    transfer.program_id = context.token_program;
    instructions.push(wrap_instruction_in_cpi_proxy(transfer)?);
    Ok(())
}

async fn add_swap_instructions<C>(
    client: &MayanClient<C>,
    quote: &Quote,
    route: &MayanSwiftQuote,
    context: &SwiftBuildContext,
    instructions: &mut Vec<Instruction>,
) -> Result<Vec<String>, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let min_middle_amount = value_to_query(route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?)?;
    let swap: SolanaClientSwap = client
        .get_swap(
            "/get-swap/solana",
            GetSwapSolanaParams::swift(
                route,
                min_middle_amount,
                context.swift_input_contract.clone(),
                quote.request.wallet_address.clone(),
                quote.from_value.clone(),
                context.referrer_address.clone(),
                context.state.to_string(),
            ),
        )
        .await?;

    instructions.extend(instructions_from_primitives(swap.compute_budget_instructions.unwrap_or_default()).map_err(solana_error)?);
    instructions.extend(setup_instructions(swap.setup_instructions.unwrap_or_default(), &context.relayer)?);
    instructions.push(instruction_from_primitive(swap.swap_instruction).map_err(solana_error)?);
    if let Some(cleanup_instruction) = swap.cleanup_instruction {
        instructions.push(wrap_instruction_in_cpi_proxy(instruction_from_primitive(cleanup_instruction).map_err(solana_error)?)?);
    }
    Ok(swap.address_lookup_table_addresses)
}

fn create_custom_payload_account(instructions: &mut Vec<Instruction>, relayer: &Pubkey, custom_payload: Option<&[u8]>) -> Result<Option<(Pubkey, u16)>, SwapperError> {
    let Some(custom_payload) = custom_payload else {
        return Ok(None);
    };
    let nonce = rand::rng().random::<u16>();
    let nonce_bytes = nonce.to_le_bytes();
    let payload_writer = SolanaAddress::parse(MAYAN_PAYLOAD_WRITER_PROGRAM_ID).map_err(solana_error)?.into();
    let seeds: [&[u8]; 3] = [b"PAYLOAD", relayer.as_bytes(), &nonce_bytes];
    let (payload_account, _) = find_program_address(&payload_writer, &seeds).map_err(solana_error)?;
    instructions.push(wrap_instruction_in_cpi_proxy(create_payload_writer_create_instruction(
        relayer,
        &payload_account,
        custom_payload,
        nonce,
    )?)?);
    Ok(Some((payload_account, nonce)))
}

fn setup_instructions(instructions: Vec<SolanaInstruction>, payer: &Pubkey) -> Result<Vec<Instruction>, SwapperError> {
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

fn wrap_instruction_in_cpi_proxy(instruction: Instruction) -> Result<Instruction, SwapperError> {
    let mut accounts = Vec::with_capacity(instruction.accounts.len() + 1);
    accounts.push(AccountMeta::new_readonly(instruction.program_id));
    accounts.extend(instruction.accounts);
    Ok(Instruction {
        program_id: SolanaAddress::parse(MAYAN_CPI_PROXY_PROGRAM_ID).map_err(solana_error)?.into(),
        accounts,
        data: instruction.data,
    })
}

fn swift_token_program(route: &MayanSwiftQuote) -> Pubkey {
    match route.swift_input_contract_standard.as_deref() {
        Some("spl2022") => program_ids::token_2022_program(),
        _ => program_ids::token_program(),
    }
}
