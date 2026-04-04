use primitives::SignerError;
use signer::decode_base32;

pub(crate) use signer::Base32Address;

const ED25519_PUBLIC_KEY_VERSION: u8 = 0x30;
const ADDRESS_LENGTH: usize = 56;
const DECODED_ADDRESS_LENGTH: usize = 35;

pub(crate) fn parse_address(value: &str) -> Result<Base32Address, SignerError> {
    if value.len() != ADDRESS_LENGTH || !value.starts_with('G') {
        return Err(SignerError::invalid_input("invalid Stellar address"));
    }

    let decoded = decode_base32(value.as_bytes()).ok_or_else(|| SignerError::invalid_input("invalid Stellar address"))?;
    if decoded.len() != DECODED_ADDRESS_LENGTH || decoded[0] != ED25519_PUBLIC_KEY_VERSION {
        return Err(SignerError::invalid_input("invalid Stellar address"));
    }

    let expected_checksum = u16::from_le_bytes([decoded[33], decoded[34]]);
    if crc16_xmodem(&decoded[..33]) != expected_checksum {
        return Err(SignerError::invalid_input("invalid Stellar address checksum"));
    }

    Base32Address::from_slice(&decoded[1..33])
}

fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}
