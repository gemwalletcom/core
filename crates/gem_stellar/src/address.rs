use primitives::SignerError;
use signer::decode_base32;

pub(crate) use signer::Base32Address;

const ED25519_PUBLIC_KEY_VERSION: u8 = 0x30;
const ADDRESS_LENGTH: usize = 56;
const DECODED_ADDRESS_LENGTH: usize = 35;
const INVALID_ADDRESS: &str = "invalid Stellar address";
const CRC16_XMODEM_POLY: u16 = 0x1021;
const CRC16_XMODEM_MSB_MASK: u16 = 0x8000;

pub(crate) fn parse_address(value: &str) -> Result<Base32Address, SignerError> {
    let decoded = (|| -> Result<Vec<u8>, &'static str> {
        if value.len() != ADDRESS_LENGTH || !value.starts_with('G') {
            return Err(INVALID_ADDRESS);
        }

        let decoded = decode_base32(value.as_bytes()).ok_or(INVALID_ADDRESS)?;
        if decoded.len() != DECODED_ADDRESS_LENGTH || decoded[0] != ED25519_PUBLIC_KEY_VERSION {
            return Err(INVALID_ADDRESS);
        }
        Ok(decoded)
    })()
    .map_err(SignerError::from_display)?;

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
            crc = if crc & CRC16_XMODEM_MSB_MASK != 0 { (crc << 1) ^ CRC16_XMODEM_POLY } else { crc << 1 };
        }
    }
    crc
}
