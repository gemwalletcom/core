use super::{circle_max_fee64, destination_referrer_address, fast_mctp_input_contract, fast_mctp_min_finality, referrer_bytes, refund_relayer_fee64, token_out};
use crate::{
    Quote, RpcProvider, SwapperError, SwapperQuoteData,
    mayan::{
        cctp_domain::{CCTP_TOKEN_DECIMALS, domain_for_wormhole_chain},
        client::MayanClient,
        constants::{MAYAN_FAST_MCTP_PROGRAM_ID, MAYAN_LOOKUP_TABLE_SOLANA},
        model::{GetSwapSolanaParams, MayanFastMctpQuote, QuoteType, SolanaClientSwap},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{fractional_amount, gas_drop_amount, min_amount_out, optional_bps_u8, value_to_query},
            route::quote_destination_address,
            solana::{
                self as solana_builder, SolanaLedgerDeposit, SolanaTransaction, append_client_swap_instructions, append_ledger_deposit_instructions, solana_error,
                wrap_instruction_in_cpi_proxy,
            },
        },
        wormhole_chain::{WormholeChain, id_for_name as wormhole_chain_id},
    },
};
use gem_client::Client;
use gem_solana::SolanaAddress;
use rand::Rng;
use solana_primitives::anchor::global_discriminator;
use solana_primitives::associated_token::get_associated_token_address_with_program_id;
use solana_primitives::instructions::program_ids;
use solana_primitives::{AccountMeta, Instruction, Pubkey, find_program_address};
use std::{fmt::Debug, sync::Arc};

const LEDGER_ORDER_SEED: &[u8] = b"LEDGER_ORDER";
const LEDGER_BRIDGE_SEED: &[u8] = b"LEDGER_BRIDGE";
const FAST_MCTP_MODE_BRIDGE: u8 = 1;
const FAST_MCTP_MODE_ORDER: u8 = 2;

struct FastMctpBuildContext {
    user: Pubkey,
    relayer: Pubkey,
    ledger: Pubkey,
    ledger_account: Pubkey,
    random_key: u16,
    fast_mctp_input_mint: Pubkey,
    fast_mctp_input_contract: String,
    destination_address: String,
    token_out: [u8; 32],
    referrer_address: Option<String>,
}

impl FastMctpBuildContext {
    fn new(quote: &Quote, route: &MayanFastMctpQuote) -> Result<Self, SwapperError> {
        if route.to_chain == WormholeChain::Solana.name() || route.to_chain == WormholeChain::Sui.name() {
            return Err(SwapperError::InvalidRoute);
        }

        let wallet_address = quote.request.wallet_address.as_str();
        let relayer_address = route.relayer.as_deref().unwrap_or(wallet_address);
        if relayer_address != wallet_address {
            return Err(SwapperError::InvalidRoute);
        }

        let user: Pubkey = SolanaAddress::parse(wallet_address).map_err(solana_error)?.into();
        let relayer: Pubkey = SolanaAddress::parse(relayer_address).map_err(solana_error)?.into();
        let fast_mctp_program: Pubkey = SolanaAddress::parse(MAYAN_FAST_MCTP_PROGRAM_ID).map_err(solana_error)?.into();
        let fast_mctp_input_contract = fast_mctp_input_contract(route)?.to_string();
        let fast_mctp_input_mint: Pubkey = SolanaAddress::parse(&fast_mctp_input_contract).map_err(solana_error)?.into();
        let random_key = random_u16();
        let seed_prefix = if route.has_auction == Some(true) { LEDGER_ORDER_SEED } else { LEDGER_BRIDGE_SEED };
        let (ledger, _) = find_program_address(&fast_mctp_program, &[seed_prefix, user.as_bytes(), &random_key.to_le_bytes()]).map_err(solana_error)?;
        let ledger_account = get_associated_token_address_with_program_id(&ledger, &fast_mctp_input_mint, &program_ids::token_program());
        let destination_address = quote_destination_address(quote).to_string();
        let token_out = token_out(route)?;
        let referrer_address = destination_referrer_address(route)?;

        Ok(Self {
            user,
            relayer,
            ledger,
            ledger_account,
            random_key,
            fast_mctp_input_mint,
            fast_mctp_input_contract,
            destination_address,
            token_out,
            referrer_address,
        })
    }
}

pub async fn build_quote_data<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanFastMctpQuote, rpc_provider: Arc<dyn RpcProvider>) -> Result<SwapperQuoteData, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let transaction = build(client, quote, route).await?;
    solana_builder::build_quote_data(quote, transaction, rpc_provider).await
}

async fn build<C>(client: &MayanClient<C>, quote: &Quote, route: &MayanFastMctpQuote) -> Result<SolanaTransaction, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let context = FastMctpBuildContext::new(quote, route)?;
    let mut instructions = Vec::new();
    let mut lookup_table_addresses = vec![MAYAN_LOOKUP_TABLE_SOLANA.to_string()];

    if route.from_token.contract.as_str() == context.fast_mctp_input_contract.as_str() {
        add_direct_fast_mctp_instructions(route, &context, &mut instructions)?;
    } else {
        lookup_table_addresses.extend(add_swap_instructions(client, quote, route, &context, &mut instructions).await?);
    }

    Ok(SolanaTransaction::new(instructions, lookup_table_addresses))
}

fn add_direct_fast_mctp_instructions(route: &MayanFastMctpQuote, context: &FastMctpBuildContext, instructions: &mut Vec<Instruction>) -> Result<(), SwapperError> {
    let amount = route.effective_amount_in64.parse::<u64>()?;
    append_ledger_deposit_instructions(
        instructions,
        SolanaLedgerDeposit {
            user: &context.user,
            relayer: &context.relayer,
            ledger: &context.ledger,
            ledger_account: &context.ledger_account,
            mint: &context.fast_mctp_input_mint,
            amount,
            suggested_priority_fee: route.suggested_priority_fee,
        },
    )?;

    add_ledger_instruction(route, context, amount, solana_relayer_fee(route)?, instructions)
}

async fn add_swap_instructions<C>(
    client: &MayanClient<C>,
    quote: &Quote,
    route: &MayanFastMctpQuote,
    context: &FastMctpBuildContext,
    instructions: &mut Vec<Instruction>,
) -> Result<Vec<String>, SwapperError>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    let min_middle_amount = route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?;
    let deposit_mode = if route.has_auction == Some(true) { "FAST_MCTP_ORDER" } else { "FAST_MCTP_BRIDGE" };
    let swap: SolanaClientSwap = client
        .get_swap(
            "/get-swap/solana",
            GetSwapSolanaParams::fast_mctp(
                route,
                value_to_query(min_middle_amount)?,
                context.fast_mctp_input_contract.clone(),
                quote.request.wallet_address.clone(),
                route.effective_amount_in64.clone(),
                deposit_mode,
                context.referrer_address.clone(),
                context.ledger.to_string(),
            ),
        )
        .await?;

    let lookup_table_addresses = append_client_swap_instructions(
        instructions,
        swap,
        &context.user,
        &context.relayer,
        &route.from_token.contract,
        &route.effective_amount_in64,
    )?;

    add_ledger_instruction(
        route,
        context,
        fractional_amount(min_middle_amount, CCTP_TOKEN_DECIMALS)?,
        solana_relayer_fee(route)?,
        instructions,
    )?;

    Ok(lookup_table_addresses)
}

fn add_ledger_instruction(
    route: &MayanFastMctpQuote,
    context: &FastMctpBuildContext,
    amount_in_min64: u64,
    fee_solana: u64,
    instructions: &mut Vec<Instruction>,
) -> Result<(), SwapperError> {
    if route.has_auction == Some(true) {
        instructions.push(wrap_instruction_in_cpi_proxy(create_fast_mctp_order_ledger_instruction(
            route,
            context,
            amount_in_min64,
            fee_solana,
        )?)?);
    } else {
        instructions.push(wrap_instruction_in_cpi_proxy(create_fast_mctp_bridge_ledger_instruction(
            route,
            context,
            amount_in_min64,
            fee_solana,
        )?)?);
    }
    Ok(())
}

fn create_fast_mctp_bridge_ledger_instruction(
    route: &MayanFastMctpQuote,
    context: &FastMctpBuildContext,
    amount_in_min64: u64,
    fee_solana: u64,
) -> Result<Instruction, SwapperError> {
    let fast_mctp_program = SolanaAddress::parse(MAYAN_FAST_MCTP_PROGRAM_ID).map_err(solana_error)?.into();
    let data = bridge_ledger_data(route, context, amount_in_min64, fee_solana)?;
    Ok(Instruction {
        program_id: fast_mctp_program,
        accounts: vec![
            AccountMeta::new_signer_writable(context.user),
            AccountMeta::new_writable(context.ledger),
            AccountMeta::new_signer_writable(context.relayer),
            AccountMeta::new_readonly(context.ledger_account),
            AccountMeta::new_readonly(fast_mctp_program),
            AccountMeta::new_readonly(context.fast_mctp_input_mint),
            AccountMeta::new_readonly(program_ids::system_program()),
        ],
        data,
    })
}

fn create_fast_mctp_order_ledger_instruction(
    route: &MayanFastMctpQuote,
    context: &FastMctpBuildContext,
    amount_in_min64: u64,
    fee_solana: u64,
) -> Result<Instruction, SwapperError> {
    let fast_mctp_program = SolanaAddress::parse(MAYAN_FAST_MCTP_PROGRAM_ID).map_err(solana_error)?.into();
    let mut data = Vec::with_capacity(159);
    data.extend_from_slice(&global_discriminator("init_order_ledger"));
    data.extend_from_slice(&destination_address(route, context)?);
    data.extend_from_slice(&amount_in_min64.to_le_bytes());
    data.extend_from_slice(&gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?.to_le_bytes());
    data.extend_from_slice(&redeem_relayer_fee64(route)?.to_le_bytes());
    data.extend_from_slice(&refund_relayer_fee64(route)?.to_le_bytes());
    data.extend_from_slice(&fee_solana.to_le_bytes());
    data.extend_from_slice(&domain_for_wormhole_chain(&route.to_chain)?.id().to_le_bytes());
    data.extend_from_slice(&context.random_key.to_le_bytes());
    data.push(FAST_MCTP_MODE_ORDER);
    data.extend_from_slice(&context.token_out);
    data.extend_from_slice(&min_amount_out(&route.min_amount_out, route.to_token.decimals, &route.to_chain, &QuoteType::FastMctp)?.to_le_bytes());
    data.extend_from_slice(&route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&referrer_bytes(route)?);
    data.push(optional_bps_u8(route.referrer_bps)?);
    data.extend_from_slice(&circle_max_fee64(route)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&fast_mctp_min_finality(route)?.to_le_bytes());

    Ok(Instruction {
        program_id: fast_mctp_program,
        accounts: vec![
            AccountMeta::new_signer_writable(context.user),
            AccountMeta::new_writable(context.ledger),
            AccountMeta::new_signer_writable(context.relayer),
            AccountMeta::new_readonly(context.ledger_account),
            AccountMeta::new_readonly(context.fast_mctp_input_mint),
            AccountMeta::new_readonly(program_ids::system_program()),
        ],
        data,
    })
}

fn bridge_ledger_data(route: &MayanFastMctpQuote, context: &FastMctpBuildContext, amount_in_min64: u64, fee_solana: u64) -> Result<Vec<u8>, SwapperError> {
    let mut data = Vec::with_capacity(124);
    data.extend_from_slice(&global_discriminator("init_bridge_ledger"));
    data.extend_from_slice(&destination_address(route, context)?);
    data.extend_from_slice(&amount_in_min64.to_le_bytes());
    data.extend_from_slice(&gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::FastMctp, false)?.to_le_bytes());
    data.extend_from_slice(&redeem_relayer_fee64(route)?.to_le_bytes());
    data.extend_from_slice(&fee_solana.to_le_bytes());
    data.extend_from_slice(&domain_for_wormhole_chain(&route.to_chain)?.id().to_le_bytes());
    data.extend_from_slice(&referrer_bytes(route)?);
    data.push(optional_bps_u8(route.referrer_bps)?);
    data.extend_from_slice(&context.random_key.to_le_bytes());
    data.extend_from_slice(&circle_max_fee64(route)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&fast_mctp_min_finality(route)?.to_le_bytes());
    data.push(FAST_MCTP_MODE_BRIDGE);
    Ok(data)
}

fn destination_address(route: &MayanFastMctpQuote, context: &FastMctpBuildContext) -> Result<[u8; 32], SwapperError> {
    native_address_to_bytes32(&context.destination_address, wormhole_chain_id(&route.to_chain)?)
}

fn redeem_relayer_fee64(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    route
        .redeem_relayer_fee64
        .as_deref()
        .ok_or(SwapperError::InvalidRoute)?
        .parse::<u64>()
        .map_err(SwapperError::from)
}

fn solana_relayer_fee(route: &MayanFastMctpQuote) -> Result<u64, SwapperError> {
    route
        .solana_relayer_fee64
        .as_deref()
        .ok_or(SwapperError::InvalidRoute)?
        .parse::<u64>()
        .map_err(SwapperError::from)
}

fn random_u16() -> u16 {
    let mut bytes = [0u8; 2];
    rand::rng().fill_bytes(&mut bytes);
    u16::from_le_bytes(bytes) % 65000
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, asset_constants::SOLANA_USDC_TOKEN_ID};

    #[test]
    fn test_create_fast_mctp_bridge_ledger_instruction() {
        let mut quote = Quote::mock(Chain::Solana, Some(SOLANA_USDC_TOKEN_ID));
        quote.request.wallet_address = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string();
        quote.request.destination_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        let route = MayanFastMctpQuote::mock();
        let context = FastMctpBuildContext::new(&quote, &route).unwrap();
        let instruction = create_fast_mctp_bridge_ledger_instruction(&route, &context, 1_000_000, 179_182).unwrap();

        assert_eq!(instruction.program_id.to_string(), MAYAN_FAST_MCTP_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 7);
        assert_eq!(instruction.accounts[4].pubkey.to_string(), MAYAN_FAST_MCTP_PROGRAM_ID);
        assert_eq!(instruction.accounts[5].pubkey.to_string(), SOLANA_USDC_TOKEN_ID);
        assert_eq!(&instruction.data[..8], global_discriminator("init_bridge_ledger").as_slice());
        assert_eq!(u32::from_le_bytes(instruction.data[72..76].try_into().unwrap()), 6);
        assert_eq!(instruction.data[123], FAST_MCTP_MODE_BRIDGE);
    }

    #[test]
    fn test_create_fast_mctp_order_ledger_instruction_uses_order_mode() {
        let mut quote = Quote::mock(Chain::Solana, Some(SOLANA_USDC_TOKEN_ID));
        quote.request.wallet_address = "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string();
        quote.request.destination_address = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string();
        let mut route = MayanFastMctpQuote::mock();
        route.has_auction = Some(true);
        let context = FastMctpBuildContext::new(&quote, &route).unwrap();
        let instruction = create_fast_mctp_order_ledger_instruction(&route, &context, 1_000_000, 179_182).unwrap();

        assert_eq!(instruction.program_id.to_string(), MAYAN_FAST_MCTP_PROGRAM_ID);
        assert_eq!(instruction.accounts.len(), 6);
        assert_eq!(&instruction.data[..8], global_discriminator("init_order_ledger").as_slice());
        assert_eq!(u32::from_le_bytes(instruction.data[80..84].try_into().unwrap()), 6);
        assert_eq!(instruction.data[86], FAST_MCTP_MODE_ORDER);
    }
}
