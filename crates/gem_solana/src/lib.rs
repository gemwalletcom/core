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

#[cfg(feature = "signer")]
pub mod signer;

pub use jsonrpc::SolanaRpc;

#[cfg(all(feature = "reqwest", not(feature = "rpc")))]
pub use rpc::client::SolanaClient;

// Constants
pub use primitives::asset_constants::{
    SOLANA_PYUSD_TOKEN_ID as PYUSD_TOKEN_MINT, SOLANA_USDC_TOKEN_ID as USDC_TOKEN_MINT, SOLANA_USDS_TOKEN_ID as USDS_TOKEN_MINT, SOLANA_USDT_TOKEN_ID as USDT_TOKEN_MINT,
};
pub use primitives::contract_constants::{
    SOLANA_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM, SOLANA_BPF_LOADER_PROGRAM_ID as BPF_LOADER_PROGRAM_ID,
    SOLANA_COMPUTE_BUDGET_PROGRAM_ID as COMPUTE_BUDGET_PROGRAM_ID, SOLANA_JITO_TIP_PROGRAM_ID as JITO_TIP_PROGRAM_ID, SOLANA_JUPITER_PROGRAM_ID as JUPITER_PROGRAM_ID,
    SOLANA_MEMO_PROGRAM_ID as MEMO_PROGRAM_ID, SOLANA_METAPLEX_PROGRAM_ID as METAPLEX_PROGRAM, SOLANA_OKX_DEX_V2_PROGRAM_ID as OKX_DEX_V2_PROGRAM_ID,
    SOLANA_STAKE_PROGRAM_ID as STAKE_PROGRAM_ID, SOLANA_SYSTEM_PROGRAM_ID as SYSTEM_PROGRAM_ID, SOLANA_SYSVAR_CLOCK_ID as SYSVAR_CLOCK_ID,
    SOLANA_SYSVAR_INSTRUCTIONS_ID as SYSVAR_INSTRUCTIONS_ID, SOLANA_SYSVAR_RENT_ID as SYSVAR_RENT_ID, SOLANA_TOKEN_2022_PROGRAM_ID as TOKEN_PROGRAM_2022,
    SOLANA_TOKEN_PROGRAM_ID as TOKEN_PROGRAM, SOLANA_VOTE_PROGRAM_ID as VOTE_PROGRAM_ID, SOLANA_WRAPPED_SOL_TOKEN_ADDRESS as WSOL_TOKEN_ADDRESS,
};
pub const COMPUTE_UNIT_LIMIT_DISCRIMINANT: u8 = 2;
pub const COMPUTE_UNIT_PRICE_DISCRIMINANT: u8 = 3;
pub const SYSTEM_PROGRAMS: &[&str] = &[
    SYSTEM_PROGRAM_ID,
    COMPUTE_BUDGET_PROGRAM_ID,
    TOKEN_PROGRAM,
    TOKEN_PROGRAM_2022,
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM,
    VOTE_PROGRAM_ID,
    STAKE_PROGRAM_ID,
    SYSVAR_CLOCK_ID,
    SYSVAR_RENT_ID,
    SYSVAR_INSTRUCTIONS_ID,
    BPF_LOADER_PROGRAM_ID,
    MEMO_PROGRAM_ID,
    JITO_TIP_PROGRAM_ID,
];
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
