pub mod hash;
pub mod jsonrpc;
pub mod metaplex;
pub mod pubkey;

pub const WSOL_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDC_TOKEN_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_TOKEN_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

pub const METAPLEX_PROGRAM: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_PROGRAM_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

use primitives::{AssetId, SolanaTokenProgramId};
use pubkey::Pubkey;
use std::str::FromStr;

pub fn get_token_program_by_id(id: SolanaTokenProgramId) -> &'static str {
    match id {
        SolanaTokenProgramId::Token => TOKEN_PROGRAM,
        SolanaTokenProgramId::Token2022 => TOKEN_PROGRAM_2022,
    }
}

pub fn get_token_program_id_by_address(address: &str) -> Option<SolanaTokenProgramId> {
    if address == TOKEN_PROGRAM {
        Some(SolanaTokenProgramId::Token)
    } else if address == TOKEN_PROGRAM_2022 {
        Some(SolanaTokenProgramId::Token2022)
    } else {
        None
    }
}

pub fn get_pubkey_by_asset(asset_id: &AssetId) -> Option<Pubkey> {
    match &asset_id.token_id {
        Some(token_id) => Pubkey::from_str(token_id).ok(),
        None => Pubkey::from_str(WSOL_TOKEN_ADDRESS).ok(),
    }
}

pub fn get_pubkey_by_str(asset_id: &str) -> Option<Pubkey> {
    let asset_id = AssetId::new(asset_id)?;
    get_pubkey_by_asset(&asset_id)
}
