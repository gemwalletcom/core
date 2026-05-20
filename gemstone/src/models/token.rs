use primitives::solana_nft::SolanaNftStandard;
use primitives::solana_token_program::SolanaTokenProgramId;

pub type GemSolanaTokenProgramId = SolanaTokenProgramId;
pub type GemSolanaNftStandard = SolanaNftStandard;

#[uniffi::remote(Enum)]
pub enum SolanaTokenProgramId {
    Token,
    Token2022,
}

#[uniffi::remote(Enum)]
pub enum SolanaNftStandard {
    NonFungible,
    ProgrammableNonFungible { rule_set: Option<String> },
    Core { collection: Option<String> },
}
