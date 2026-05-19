use super::solana_error;
use crate::{
    SwapperError,
    mayan::{
        constants::{MAYAN_FEE_MANAGER_PROGRAM_ID, MAYAN_SWIFT_V2_PROGRAM_ID},
        model::{MayanSwiftQuote, QuoteType},
        tx_builder::{
            address::native_address_to_bytes32,
            amount::{bps_u8, fractional_amount, gas_drop_amount, min_amount_out},
            route::{is_hypercore_deposit, swift_destination_chain, swift_destination_chain_id},
            swift::{referrer_bytes, swift_custom_payload_hash, swift_payload_type, swift_to_token, token_address_to_bytes32},
        },
        wormhole_chain::id_for_name as wormhole_chain_id,
    },
};
use gem_evm::EVM_ZERO_ADDRESS;
use gem_hash::keccak::keccak256;
use gem_solana::{SYSTEM_PROGRAM_ID, SolanaAddress};
use solana_primitives::anchor::global_discriminator;
use solana_primitives::{AccountMeta, Instruction, Pubkey};

const SWIFT_ORDER_DATA_SIZE_V2: usize = 272;

pub(super) fn create_init_instruction(
    route: &MayanSwiftQuote,
    state: &Pubkey,
    trader: &Pubkey,
    relayer: &Pubkey,
    state_account: &Pubkey,
    relayer_account: &Pubkey,
    swift_input_mint: &Pubkey,
    token_program: &Pubkey,
    destination_address: &str,
    random_key: [u8; 32],
    custom_payload_account: Option<&Pubkey>,
) -> Result<Instruction, SwapperError> {
    let destination_chain_id = swift_destination_chain_id(route)?;
    if !is_hypercore_deposit(route) && destination_chain_id != route.to_token.w_chain_id {
        return Err(SwapperError::InvalidRoute);
    }
    let swift_input_contract = route.swift_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let amount_in_min = if route.from_token.contract.as_str() == swift_input_contract {
        route.effective_amount_in64.parse::<u64>()?
    } else {
        fractional_amount::<u64>(
            route.min_middle_amount.as_ref().ok_or(SwapperError::InvalidRoute)?,
            route.swift_input_decimals.ok_or(SwapperError::InvalidRoute)?,
        )?
    };
    let referrer = referrer_bytes(&route.from_chain)?;
    let destination_chain = swift_destination_chain(route);
    let amount_out_min = min_amount_out(&route.min_amount_out, route.to_token.decimals, destination_chain, &QuoteType::Swift)?;
    let gas_drop = gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Swift, is_hypercore_deposit(route))?;

    let mut data = Vec::with_capacity(8 + 8 + 1 + 8 + 32 + 2 + 32 + 8 + 8 + 8 + 8 + 8 + 32 + 1 + 1 + 1 + 32);
    data.extend_from_slice(&global_discriminator("init_order"));
    data.extend_from_slice(&amount_in_min.to_le_bytes());
    data.push(u8::from(swift_input_contract == EVM_ZERO_ADDRESS));
    data.extend_from_slice(&route.submit_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&native_address_to_bytes32(destination_address, destination_chain_id)?);
    data.extend_from_slice(&destination_chain_id.to_le_bytes());
    data.extend_from_slice(&swift_to_token(route)?);
    data.extend_from_slice(&amount_out_min.to_le_bytes());
    data.extend_from_slice(&gas_drop.to_le_bytes());
    data.extend_from_slice(&route.cancel_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&route.refund_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_le_bytes());
    data.extend_from_slice(&referrer);
    data.push(bps_u8(route.referrer_bps)?);
    data.push(bps_u8(route.protocol_bps)?);
    data.push(route.swift_auction_mode.ok_or(SwapperError::InvalidRoute)?);
    data.extend_from_slice(&random_key);

    let swift_program = SolanaAddress::parse(MAYAN_SWIFT_V2_PROGRAM_ID).map_err(solana_error)?.into();
    let fee_manager = SolanaAddress::parse(MAYAN_FEE_MANAGER_PROGRAM_ID).map_err(solana_error)?.into();
    let system_program = SolanaAddress::parse(SYSTEM_PROGRAM_ID).map_err(solana_error)?.into();

    Ok(Instruction {
        program_id: swift_program,
        accounts: vec![
            AccountMeta::new_readonly(*trader),
            AccountMeta::new(*relayer, true, true),
            AccountMeta::new_writable(*state),
            AccountMeta::new_writable(*state_account),
            AccountMeta::new_writable(*relayer_account),
            AccountMeta::new_readonly(custom_payload_account.copied().unwrap_or(swift_program)),
            AccountMeta::new_readonly(*swift_input_mint),
            AccountMeta::new_readonly(fee_manager),
            AccountMeta::new_readonly(*token_program),
            AccountMeta::new_readonly(system_program),
        ],
        data,
    })
}

pub(super) fn create_order_hash(
    route: &MayanSwiftQuote,
    swapper_address: &str,
    destination_address: &str,
    random_key: &[u8; 32],
    custom_payload: Option<&[u8]>,
) -> Result<[u8; 32], SwapperError> {
    let source_chain_id = wormhole_chain_id(&route.from_chain)?;
    let destination_chain_id = swift_destination_chain_id(route)?;
    let destination_chain = swift_destination_chain(route);
    let swift_input_contract = route.swift_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let token_in = token_address_to_bytes32(swift_input_contract, &route.from_chain)?;
    let referrer = referrer_bytes(&route.from_chain)?;
    let amount_out_min = min_amount_out(&route.min_amount_out, route.to_token.decimals, destination_chain, &QuoteType::Swift)?;
    let gas_drop = gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Swift, is_hypercore_deposit(route))?;

    let mut data = Vec::with_capacity(SWIFT_ORDER_DATA_SIZE_V2);
    data.push(swift_payload_type(custom_payload));
    data.extend_from_slice(&native_address_to_bytes32(swapper_address, source_chain_id)?);
    data.extend_from_slice(&source_chain_id.to_be_bytes());
    data.extend_from_slice(&token_in);
    data.extend_from_slice(&native_address_to_bytes32(destination_address, destination_chain_id)?);
    data.extend_from_slice(&destination_chain_id.to_be_bytes());
    data.extend_from_slice(&swift_to_token(route)?);
    data.extend_from_slice(&amount_out_min.to_be_bytes());
    data.extend_from_slice(&gas_drop.to_be_bytes());
    data.extend_from_slice(&route.cancel_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_be_bytes());
    data.extend_from_slice(&route.refund_relayer_fee64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_be_bytes());
    data.extend_from_slice(&route.deadline64.as_deref().ok_or(SwapperError::InvalidRoute)?.parse::<u64>()?.to_be_bytes());
    data.extend_from_slice(&referrer);
    data.push(bps_u8(route.referrer_bps)?);
    data.push(bps_u8(route.protocol_bps)?);
    data.push(route.swift_auction_mode.ok_or(SwapperError::InvalidRoute)?);
    data.extend_from_slice(random_key);
    data.extend_from_slice(&swift_custom_payload_hash(custom_payload)?);

    if data.len() != SWIFT_ORDER_DATA_SIZE_V2 {
        return Err(SwapperError::InvalidRoute);
    }
    Ok(keccak256(&data))
}
