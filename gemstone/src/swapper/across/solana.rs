use borsh::{BorshDeserialize, BorshSerialize};
use solana_primitives::Pubkey;

pub const MULTICALL_HANDLER: &str = "HaQe51FWtnmaEcuYEfPA7MRCXKrtqptat4oJdJ8zV5Be";

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CompiledIx {
    pub program_id_index: u8,
    pub account_key_indexes: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct RelayData {
    pub depositor: Pubkey,
    pub recipient: Pubkey,
    pub exclusive_relayer: Pubkey,
    pub input_token: Pubkey,
    pub output_token: Pubkey,
    pub input_amount: [u8; 32],
    pub output_amount: u64,
    pub origin_chain_id: u64,
    pub deposit_id: [u8; 32],
    pub fill_deadline: u32,
    pub exclusivity_deadline: u32,
    pub message: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct AcrossPlusMessage {
    pub handler: Pubkey,
    pub read_only_len: u8,
    pub value_amount: u64,
    pub accounts: Vec<Pubkey>,
    pub handler_message: Vec<u8>,
}
