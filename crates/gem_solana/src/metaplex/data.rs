use crate::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Default, PartialEq, Eq, Debug, Clone)]
pub struct Data {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, Eq, PartialEq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}
