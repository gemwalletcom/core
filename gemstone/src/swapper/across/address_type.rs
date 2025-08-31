use alloy_primitives::Address;

use crate::swapper::SwapperError;

#[derive(Debug, Clone)]
pub(crate) enum AddressType {
    Evm(Address),
    Solana(String),
}

impl AddressType {
    pub(crate) fn to_bytes32(&self) -> Result<[u8; 32], SwapperError> {
        match self {
            AddressType::Evm(address) => {
                let mut bytes32 = [0u8; 32];
                bytes32[12..32].copy_from_slice(address.as_slice()); // Address is 20 bytes, starts at byte 12
                Ok(bytes32)
            }
            AddressType::Solana(address_str) => {
                Self::solana_address_to_bytes32(address_str)
            }
        }
    }

    pub(crate) fn solana_address_to_bytes32(solana_address: &str) -> Result<[u8; 32], SwapperError> {
        let decoded = bs58::decode(solana_address)
            .into_vec()
            .map_err(|_| SwapperError::InvalidAddress(solana_address.to_string()))?;

        if decoded.len() != 32 {
            return Err(SwapperError::InvalidAddress(solana_address.to_string()));
        }

        decoded.try_into().map_err(|_| SwapperError::InvalidAddress(solana_address.to_string()))
    }
}
