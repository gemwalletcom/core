use crate::{SwapperError, mayan::constants::MAYAN_PAYLOAD_WRITER_PROGRAM_ID, mayan::tx_builder::solana::solana_error};
use gem_solana::{SYSTEM_PROGRAM_ID, SolanaAddress};
use solana_primitives::anchor::global_discriminator;
use solana_primitives::{AccountMeta, Instruction, Pubkey};

pub(super) fn create_payload_writer_create_instruction(payer: &Pubkey, payload_account: &Pubkey, payload: &[u8], nonce: u16) -> Result<Instruction, SwapperError> {
    let mut data = Vec::with_capacity(8 + 2 + 4 + payload.len());
    data.extend_from_slice(&global_discriminator("create_simple"));
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    data.extend_from_slice(payload);
    let payload_writer = SolanaAddress::parse(MAYAN_PAYLOAD_WRITER_PROGRAM_ID).map_err(solana_error)?.into();
    let system_program = SolanaAddress::parse(SYSTEM_PROGRAM_ID).map_err(solana_error)?.into();
    Ok(Instruction {
        program_id: payload_writer,
        accounts: vec![
            AccountMeta::new_signer_writable(*payer),
            AccountMeta::new_writable(*payload_account),
            AccountMeta::new_readonly(system_program),
        ],
        data,
    })
}

pub(super) fn create_payload_writer_close_instruction(payer: &Pubkey, payload_account: &Pubkey, nonce: u16) -> Result<Instruction, SwapperError> {
    let mut data = Vec::with_capacity(8 + 2);
    data.extend_from_slice(&global_discriminator("close"));
    data.extend_from_slice(&nonce.to_le_bytes());
    let payload_writer = SolanaAddress::parse(MAYAN_PAYLOAD_WRITER_PROGRAM_ID).map_err(solana_error)?.into();
    Ok(Instruction {
        program_id: payload_writer,
        accounts: vec![AccountMeta::new_signer_writable(*payer), AccountMeta::new_writable(*payload_account)],
        data,
    })
}
