use std::str::FromStr;

use super::WHIRLPOOL_PROGRAM;
use solana_sdk::pubkey::Pubkey;

#[allow(dead_code)]
pub fn get_fee_tier_address(whirlpools_config: &Pubkey, tick_spacing: u16) -> Option<(Pubkey, u8)> {
    let seeds = &[b"fee_tier", whirlpools_config.as_ref(), &tick_spacing.to_le_bytes()];
    let whirlpool_program = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;
    Pubkey::try_find_program_address(seeds, &whirlpool_program)
}

pub fn get_whirlpool_address(whirlpools_config: &Pubkey, token_mint_a: &Pubkey, token_mint_b: &Pubkey, tick_spacing: u16) -> Option<(Pubkey, u8)> {
    let tick_spacing_bytes = tick_spacing.to_le_bytes();
    let seeds = &[
        b"whirlpool",
        whirlpools_config.as_ref(),
        token_mint_a.as_ref(),
        token_mint_b.as_ref(),
        tick_spacing_bytes.as_ref(),
    ];
    let whirlpool_program = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;
    Pubkey::try_find_program_address(seeds, &whirlpool_program)
}

pub fn get_oracle_address(whirlpool: &Pubkey) -> Option<(Pubkey, u8)> {
    let seeds = &[b"oracle", whirlpool.as_ref()];
    let whirlpool_program = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;
    Pubkey::try_find_program_address(seeds, &whirlpool_program)
}

pub fn get_tick_array_address(whirlpool: &Pubkey, start_tick_index: i32) -> Option<(Pubkey, u8)> {
    let start_tick_index_str = start_tick_index.to_string();
    let seeds = &[b"tick_array", whirlpool.as_ref(), start_tick_index_str.as_bytes()];
    let whirlpool_program = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;
    Pubkey::try_find_program_address(seeds, &whirlpool_program)
}
