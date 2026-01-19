use borsh::{BorshDeserialize, BorshSerialize};
use solana_primitives::{
    instructions::program_ids::TOKEN_PROGRAM_ID,
    types::{AccountMeta, Instruction, Pubkey},
};

pub const MULTICALL_HANDLER: &str = "HaQe51FWtnmaEcuYEfPA7MRCXKrtqptat4oJdJ8zV5Be";
pub const DEFAULT_SOLANA_COMPUTE_LIMIT: u64 = 200_000;
pub const SOL_NATIVE_DECIMALS: u32 = 9;
pub const SOL_RELAYER_FEE_LAMPORTS: u64 = 5_000;

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

/// SPL Token TransferChecked instruction with correct account order.
/// Official SPL Token account order: [source, mint, destination, authority]
/// Note: solana-primitives uses wrong order [source, destination, mint, authority]
pub fn spl_transfer_checked(source: &Pubkey, mint: &Pubkey, destination: &Pubkey, authority: &Pubkey, amount: u64, decimals: u8) -> Instruction {
    let accounts = vec![
        AccountMeta {
            pubkey: *source,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *authority,
            is_signer: true,
            is_writable: false,
        },
    ];

    let mut data = Vec::with_capacity(10);
    data.push(12); // TransferChecked instruction index
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(decimals);

    Instruction {
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts,
        data,
    }
}
