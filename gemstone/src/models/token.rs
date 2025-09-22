use primitives::solana_token_program::SolanaTokenProgramId;

pub type GemSolanaTokenProgramId = SolanaTokenProgramId;

#[uniffi::remote(Enum)]
pub enum SolanaTokenProgramId {
    Token,
    Token2022,
}