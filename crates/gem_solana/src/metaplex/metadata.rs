use std::str::FromStr;

use crate::metaplex::{
    collection::{Collection, CollectionDetails},
    data::Data,
    uses::Uses,
    Key, TokenStandard,
};
use crate::{pubkey::Pubkey, METAPLEX_PROGRAM};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, BorshDeserialize, BorshSerialize, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Account discriminator.
    pub key: Key,
    /// Address of the update authority.
    pub update_authority: Pubkey,
    /// Address of the mint.
    pub mint: Pubkey,
    /// Asset data.
    pub data: Data,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable, default is not
    pub is_mutable: bool,
    /// nonce for easy calculation of editions, if present
    pub edition_nonce: Option<u8>,
    /// Since we cannot easily change Metadata, we add the new DataV2 fields here at the end.
    pub token_standard: Option<TokenStandard>,
    /// Collection
    pub collection: Option<Collection>,
    /// Uses
    pub uses: Option<Uses>,
    /// Collection Details
    pub collection_details: Option<CollectionDetails>,
    /// Programmable Config
    pub programmable_config: Option<ProgrammableConfig>,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum ProgrammableConfig {
    V1 { rule_set: Option<Pubkey> },
}

impl Metadata {
    pub fn find_pda(mint: Pubkey) -> Option<(Pubkey, u8)> {
        let mpl_id = Pubkey::from_str(METAPLEX_PROGRAM).unwrap();
        let seeds = &["metadata".as_bytes(), mpl_id.as_ref(), mint.as_ref()];
        Pubkey::try_find_program_address(seeds, &mpl_id)
    }
}
