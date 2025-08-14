use std::time::SystemTime;

use borsh::{BorshDeserialize, BorshSerialize};
use gem_solana::pubkey::Pubkey;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct NameRecordHeader {
    pub discriminator: [u8; 8],
    pub parent_name: Pubkey,
    pub owner: Pubkey,
    pub nclass: Pubkey,
    pub expires_at: u64,
    pub created_at: u64,
    pub non_transferable: u8,
    pub padding: [u8; 79],
}

impl NameRecordHeader {
    pub const BYTE_SIZE: usize = 8 + 32 + 32 + 32 + 8 + 8 + 1 + 79;
    pub const EXPECTED_DISCRIMINATOR: [u8; 8] = [68, 72, 88, 44, 15, 167, 103, 243];

    pub fn try_from_slice(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if data.len() < Self::BYTE_SIZE {
            return Err("Invalid data length for NameRecordHeader".into());
        }

        let header: NameRecordHeader = BorshDeserialize::try_from_slice(data)?;

        if header.discriminator != Self::EXPECTED_DISCRIMINATOR {
            return Err("Invalid discriminator for NameRecordHeader".into());
        }

        Ok(header)
    }

    pub fn is_valid(&self) -> bool {
        if self.expires_at == 0 {
            return true;
        }

        let current_time = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let grace_period = 45 * 24 * 60 * 60;

        self.expires_at + grace_period > current_time
    }
}
