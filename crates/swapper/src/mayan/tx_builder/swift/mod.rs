pub(in crate::mayan) mod evm;
pub(in crate::mayan) mod solana;

use super::{
    address::native_address_to_bytes32,
    amount::{gas_drop_amount, min_amount_out, optional_bps_u8},
    route::{is_hypercore_deposit, swift_destination_chain, swift_destination_chain_id},
};
use crate::{
    SwapperError,
    fees::default_referral_address,
    mayan::{
        model::{MayanQuoteCommon, MayanSwiftQuote, QuoteType},
        wormhole_chain::{self, WormholeChain, id_for_name as wormhole_chain_id},
    },
};
use gem_evm::EVM_ZERO_ADDRESS;
use gem_hash::keccak::keccak256;
use gem_solana::SYSTEM_PROGRAM_ID;
use primitives::{asset_constants::HYPEREVM_USDC_TOKEN_ID, decode_hex};
use rand::Rng;

const SWIFT_PAYLOAD_TYPE_DEFAULT: u8 = 1;
const SWIFT_PAYLOAD_TYPE_CUSTOM_PAYLOAD: u8 = 2;

pub(super) struct SwiftOrderFields {
    pub(super) destination_chain_id: u16,
    pub(super) referrer: [u8; 32],
    pub(super) token_out: [u8; 32],
    pub(super) amount_out_min: u64,
    pub(super) gas_drop: u64,
    pub(super) cancel_fee: u64,
    pub(super) refund_fee: u64,
    pub(super) deadline: u64,
    pub(super) referrer_bps: u8,
    pub(super) auction_mode: u8,
}

impl SwiftOrderFields {
    pub(super) fn new(route: &MayanSwiftQuote) -> Result<Self, SwapperError> {
        let destination_chain_id = swift_destination_chain_id(route)?;
        if !is_hypercore_deposit(route) && route.to_token.w_chain_id != destination_chain_id {
            return Err(SwapperError::InvalidRoute);
        }

        let destination_chain = swift_destination_chain(route);
        Ok(Self {
            destination_chain_id,
            referrer: referrer_bytes(&route.from_chain)?,
            token_out: swift_to_token(route)?,
            amount_out_min: min_amount_out(&route.min_amount_out, route.to_token.decimals, destination_chain, &QuoteType::Swift)?,
            gas_drop: gas_drop_amount(&route.gas_drop, &route.to_chain, &QuoteType::Swift, is_hypercore_deposit(route))?,
            cancel_fee: required_u64(route.cancel_relayer_fee64.as_deref())?,
            refund_fee: required_u64(route.refund_relayer_fee64.as_deref())?,
            deadline: required_u64(route.deadline64.as_deref())?,
            referrer_bps: optional_bps_u8(route.referrer_bps)?,
            auction_mode: route.swift_auction_mode.ok_or(SwapperError::InvalidRoute)?,
        })
    }
}

pub(super) fn swift_input_contract(route: &MayanSwiftQuote) -> Result<&str, SwapperError> {
    route.swift_input_contract.as_deref().ok_or(SwapperError::InvalidRoute)
}

pub(super) fn swift_to_token(route: &MayanQuoteCommon) -> Result<[u8; 32], SwapperError> {
    let (address, chain) = swift_to_token_address(route)?;
    token_address_to_bytes32(address, chain)
}

pub(super) fn token_address_to_bytes32(token_address: &str, chain: &str) -> Result<[u8; 32], SwapperError> {
    let address = if chain == WormholeChain::Solana.name() && token_address == EVM_ZERO_ADDRESS {
        SYSTEM_PROGRAM_ID
    } else {
        token_address
    };
    native_address_to_bytes32(address, wormhole_chain_id(chain)?)
}

pub(super) fn referrer_bytes(chain: &str) -> Result<[u8; 32], SwapperError> {
    let referrer_chain = wormhole_chain::chain_for_name(chain)?;
    let referrer = default_referral_address(referrer_chain);
    if referrer.is_empty() {
        return Err(SwapperError::InvalidRoute);
    }
    token_address_to_bytes32(&referrer, chain)
}

fn swift_to_token_address(route: &MayanQuoteCommon) -> Result<(&str, &str), SwapperError> {
    if is_hypercore_deposit(route) {
        return Ok((HYPEREVM_USDC_TOKEN_ID, WormholeChain::Hyperevm.name()));
    }
    if route.to_chain == WormholeChain::Sui.name() {
        return Ok((route.to_token.verified_address.as_deref().ok_or(SwapperError::InvalidRoute)?, WormholeChain::Sui.name()));
    }
    Ok((&route.to_token.contract, &route.to_chain))
}

pub(super) fn swift_random_key(route: &MayanSwiftQuote) -> Result<[u8; 32], SwapperError> {
    let quote_id = route.quote_id.as_deref().ok_or(SwapperError::InvalidRoute)?;
    let id = left_pad_16(&decode_hex(quote_id)?)?;
    let memo_or_random = if let Some(memo_hex) = &route.memo_hex {
        left_pad_16(&decode_hex(memo_hex)?)?
    } else {
        let mut bytes = [0u8; 16];
        rand::rng().fill_bytes(&mut bytes);
        bytes
    };
    let mut random = [0u8; 32];
    random[..16].copy_from_slice(&id);
    random[16..].copy_from_slice(&memo_or_random);
    Ok(random)
}

pub(super) fn swift_payload_type(custom_payload: Option<&[u8]>) -> u8 {
    if custom_payload.is_some() {
        SWIFT_PAYLOAD_TYPE_CUSTOM_PAYLOAD
    } else {
        SWIFT_PAYLOAD_TYPE_DEFAULT
    }
}

pub(super) fn swift_custom_payload_hash(custom_payload: Option<&[u8]>) -> Result<[u8; 32], SwapperError> {
    if let Some(custom_payload) = custom_payload {
        Ok(keccak256(custom_payload))
    } else {
        native_address_to_bytes32(SYSTEM_PROGRAM_ID, wormhole_chain_id(WormholeChain::Solana.name())?)
    }
}

fn left_pad_16(bytes: &[u8]) -> Result<[u8; 16], SwapperError> {
    if bytes.len() > 16 {
        return Err(SwapperError::InvalidRoute);
    }
    let mut padded = [0u8; 16];
    padded[16 - bytes.len()..].copy_from_slice(bytes);
    Ok(padded)
}

fn required_u64(value: Option<&str>) -> Result<u64, SwapperError> {
    value.ok_or(SwapperError::InvalidRoute)?.parse::<u64>().map_err(SwapperError::from)
}
