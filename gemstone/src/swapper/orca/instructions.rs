use super::WHIRLPOOL_PROGRAM;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SwapV2Arguments {
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
}

pub struct SwapV2Instruction {
    pub accounts: Vec<AccountMeta>,
    pub args: SwapV2Arguments,
}

impl SwapV2Instruction {
    pub fn to_instruction(&self) -> Instruction {
        let data = borsh::to_vec(&self.args).unwrap();
        Instruction {
            program_id: Pubkey::from_str_const(WHIRLPOOL_PROGRAM),
            accounts: self.accounts.clone(),
            data,
        }
    }
}
