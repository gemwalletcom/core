use crate::{STAKE_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_CLOCK_ID, SYSVAR_RENT_ID};
use primitives::{SignerError, SignerInput};
use sha2::{Digest, Sha256};
use solana_primitives::{AccountMeta, Instruction, Pubkey};

const SYSVAR_STAKE_HISTORY_ID: &str = "SysvarStakeHistory1111111111111111111111111";
const STAKE_CONFIG_ID: &str = "StakeConfig11111111111111111111111111111111";
const DEFAULT_STAKE_ACCOUNT_SPACE: u64 = 200;
const SYSTEM_CREATE_ACCOUNT_WITH_SEED: u32 = 3;
const STAKE_INITIALIZE: u32 = 0;
const STAKE_DELEGATE: u32 = 2;
const STAKE_WITHDRAW: u32 = 4;
const STAKE_DEACTIVATE: u32 = 5;

pub(super) fn delegate_instructions(sender: Pubkey, validator: Pubkey, stake_account: Pubkey, seed: String, lamports: u64) -> Result<Vec<Instruction>, SignerError> {
    Ok(vec![
        create_with_seed_instruction(sender, stake_account, seed, lamports)?,
        initialize_instruction(stake_account, sender)?,
        delegate_instruction(stake_account, validator, sender)?,
    ])
}

pub(super) fn deactivate_instruction(stake_account: Pubkey, authority: Pubkey) -> Result<Instruction, SignerError> {
    Ok(Instruction {
        program_id: program()?,
        accounts: vec![
            AccountMeta {
                pubkey: stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_CLOCK_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: authority,
                is_signer: true,
                is_writable: false,
            },
        ],
        data: STAKE_DEACTIVATE.to_le_bytes().to_vec(),
    })
}

pub(super) fn withdraw_instruction(stake_account: Pubkey, recipient: Pubkey, authority: Pubkey, lamports: u64) -> Result<Instruction, SignerError> {
    let mut data = Vec::new();
    data.extend_from_slice(&STAKE_WITHDRAW.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());

    Ok(Instruction {
        program_id: program()?,
        accounts: vec![
            AccountMeta {
                pubkey: stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: recipient,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_CLOCK_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_STAKE_HISTORY_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: authority,
                is_signer: true,
                is_writable: false,
            },
        ],
        data,
    })
}

pub(super) fn from_blockhash(sender: &Pubkey, input: &SignerInput) -> Result<Pubkey, SignerError> {
    let seed = seed_from_blockhash(input)?;
    let stake_program = program()?;
    let mut hasher = Sha256::new();
    hasher.update(sender.as_bytes());
    hasher.update(seed.as_bytes());
    hasher.update(stake_program.as_bytes());
    Ok(Pubkey::new(hasher.finalize().into()))
}

pub(super) fn seed_from_blockhash(input: &SignerInput) -> Result<String, SignerError> {
    let block_hash = input.metadata.get_block_hash()?;
    block_hash
        .get(..block_hash.len().min(32))
        .map(String::from)
        .ok_or_else(|| SignerError::invalid_input("invalid Solana block hash"))
}

fn create_with_seed_instruction(sender: Pubkey, stake_account: Pubkey, seed: String, lamports: u64) -> Result<Instruction, SignerError> {
    let stake_program = program()?;
    let mut data = Vec::new();
    data.extend_from_slice(&SYSTEM_CREATE_ACCOUNT_WITH_SEED.to_le_bytes());
    data.extend_from_slice(sender.as_bytes());
    // Solana system instructions use bincode string encoding for seeds.
    data.extend_from_slice(&(seed.len() as u64).to_le_bytes());
    data.extend_from_slice(seed.as_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());
    data.extend_from_slice(&DEFAULT_STAKE_ACCOUNT_SPACE.to_le_bytes());
    data.extend_from_slice(stake_program.as_bytes());

    Ok(Instruction {
        program_id: Pubkey::from_base58(SYSTEM_PROGRAM_ID).map_err(SignerError::from_display)?,
        accounts: vec![
            AccountMeta {
                pubkey: sender,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: sender,
                is_signer: true,
                is_writable: false,
            },
        ],
        data,
    })
}

fn initialize_instruction(stake_account: Pubkey, authority: Pubkey) -> Result<Instruction, SignerError> {
    let mut data = Vec::new();
    data.extend_from_slice(&STAKE_INITIALIZE.to_le_bytes());
    data.extend_from_slice(authority.as_bytes());
    data.extend_from_slice(authority.as_bytes());
    data.extend_from_slice(&0i64.to_le_bytes());
    data.extend_from_slice(&0u64.to_le_bytes());
    data.extend_from_slice(Pubkey::new([0u8; 32]).as_bytes());

    Ok(Instruction {
        program_id: program()?,
        accounts: vec![
            AccountMeta {
                pubkey: stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_RENT_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
        ],
        data,
    })
}

fn delegate_instruction(stake_account: Pubkey, validator: Pubkey, authority: Pubkey) -> Result<Instruction, SignerError> {
    Ok(Instruction {
        program_id: program()?,
        accounts: vec![
            AccountMeta {
                pubkey: stake_account,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: validator,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_CLOCK_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(SYSVAR_STAKE_HISTORY_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: Pubkey::from_base58(STAKE_CONFIG_ID).map_err(SignerError::from_display)?,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: authority,
                is_signer: true,
                is_writable: false,
            },
        ],
        data: STAKE_DELEGATE.to_le_bytes().to_vec(),
    })
}

fn program() -> Result<Pubkey, SignerError> {
    Pubkey::from_base58(STAKE_PROGRAM_ID).map_err(SignerError::from_display)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::SignerInput;

    #[test]
    fn test_seed_from_blockhash() {
        let valid_block_hash = "1".repeat(44);
        assert_eq!(seed_from_blockhash(&SignerInput::mock_solana(&valid_block_hash)).unwrap(), "1".repeat(32));

        let invalid_block_hash = format!("{}é", "1".repeat(31));
        assert_eq!(
            seed_from_blockhash(&SignerInput::mock_solana(&invalid_block_hash)).unwrap_err().to_string(),
            "Invalid input: invalid Solana block hash"
        );
    }
}
