pub mod constants;
pub mod hash;
pub mod jsonrpc;
pub mod metaplex;
pub mod pubkey;
pub mod token_account;

#[cfg(any(feature = "rpc", feature = "reqwest"))]
pub mod rpc;

#[cfg(feature = "rpc")]
pub mod provider;

pub mod models;

pub use jsonrpc::SolanaRpc;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub use rpc::client::SolanaClient;

// Constants
pub const WSOL_TOKEN_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDC_TOKEN_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_TOKEN_MINT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDS_TOKEN_MINT: &str = "USDSwr9ApdHk5bvJKMjzff41FfuX8bSxdKcR81vTwcA";
pub const PYUSD_TOKEN_MINT: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";

// Program IDs
pub const METAPLEX_PROGRAM: &str = "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s";
pub const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_PROGRAM_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const ASSOCIATED_TOKEN_ACCOUNT_PROGRAM: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";
pub const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
pub const JUPITER_PROGRAM_ID: &str = "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4";
pub const COMMITMENT_CONFIRMED: &str = "confirmed";

use primitives::{AssetId, SolanaTokenProgramId};
use pubkey::Pubkey;
use std::str::FromStr;

pub fn get_token_program_by_id(id: SolanaTokenProgramId) -> &'static str {
    match id {
        SolanaTokenProgramId::Token => TOKEN_PROGRAM,
        SolanaTokenProgramId::Token2022 => TOKEN_PROGRAM_2022,
    }
}

pub fn get_token_program_id_by_address(address: &str) -> Result<SolanaTokenProgramId, Box<dyn std::error::Error + Send + Sync>> {
    if address == TOKEN_PROGRAM {
        Ok(SolanaTokenProgramId::Token)
    } else if address == TOKEN_PROGRAM_2022 {
        Ok(SolanaTokenProgramId::Token2022)
    } else {
        Err(format!("Unknown token program address: {}", address).into())
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
