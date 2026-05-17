use borsh::{BorshDeserialize, BorshSerialize};
use solana_primitives::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct Collection {
    pub verified: bool,
    pub key: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum CollectionDetails {
    V1 { size: u64 },
    V2 { padding: [u8; 8] },
}
