use std::str::FromStr;

use super::WHIRLPOOL_PROGRAM;
use gem_solana::pubkey::Pubkey;

#[allow(dead_code)]
pub fn get_fee_tier_address(whirlpools_config: &Pubkey, tick_spacing: u16) -> Option<(Pubkey, u8)> {
    let seeds = &[b"fee_tier", whirlpools_config.as_ref(), &tick_spacing.to_le_bytes()];
    let whirlpool_address = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;
    Pubkey::try_find_program_address(seeds, &whirlpool_address)
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

    let whirlpool_address = Pubkey::from_str(WHIRLPOOL_PROGRAM).ok()?;

    Pubkey::try_find_program_address(seeds, &whirlpool_address)
}
