use crate::{SignatureScheme, SignerError};
use primitives::hex::encode_with_0x;
use primitives::{Chain, ChainType, decode_hex};
use zeroize::Zeroizing;

#[derive(Debug, Clone, Copy)]
enum KeyEncoding {
    Hex,
    Base58,
    Base32,
}

fn import_encodings_for_chain(chain: &Chain) -> &'static [KeyEncoding] {
    match chain.chain_type() {
        ChainType::Bitcoin => &[],
        ChainType::Solana => &[KeyEncoding::Base58, KeyEncoding::Hex],
        ChainType::Stellar => &[KeyEncoding::Base32, KeyEncoding::Hex],
        _ => &[KeyEncoding::Hex],
    }
}

fn export_encoding_for_chain(chain: &Chain) -> KeyEncoding {
    match chain.chain_type() {
        ChainType::Bitcoin | ChainType::Solana => KeyEncoding::Base58,
        _ => KeyEncoding::Hex,
    }
}

pub fn supports_private_key_import(chain: &Chain) -> bool {
    !import_encodings_for_chain(chain).is_empty()
}

fn scheme_for_chain(chain: &Chain) -> SignatureScheme {
    match chain.chain_type() {
        ChainType::Solana
        | ChainType::Ton
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano => SignatureScheme::Ed25519,
        _ => SignatureScheme::Secp256k1,
    }
}

fn decode_base58(value: &str) -> Option<Vec<u8>> {
    let decoded = bs58::decode(value).into_vec().ok()?;
    match decoded.len() {
        32 => Some(decoded),
        64 => Some(decoded[..32].to_vec()),
        _ => None,
    }
}

fn base32_decode_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'2'..=b'7' => Some(c - b'2' + 26),
        _ => None,
    }
}

fn base32_decode(input: &[u8]) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(input.len() * 5 / 8);
    let mut buffer: u64 = 0;
    let mut bits = 0u8;

    for &c in input {
        if c == b'=' {
            break;
        }
        let val = base32_decode_char(c)?;
        buffer = (buffer << 5) | val as u64;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1u64 << bits) - 1;
        }
    }
    Some(output)
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

fn decode_base32_stellar(value: &str) -> Option<Vec<u8>> {
    if value.len() != 56 || !value.starts_with('S') {
        return None;
    }
    let decoded = base32_decode(value.as_bytes())?;
    if decoded.len() != 35 || decoded[0] != 0x90 {
        return None;
    }
    let expected = u16::from_le_bytes([decoded[33], decoded[34]]);
    if crc16_xmodem(&decoded[..33]) != expected {
        return None;
    }
    Some(decoded[1..33].to_vec())
}

fn validate_key(bytes: &[u8], scheme: SignatureScheme) -> Result<(), SignerError> {
    match scheme {
        SignatureScheme::Ed25519 => {
            let arr: &[u8; 32] = bytes.try_into().map_err(|_| SignerError::invalid_input("Invalid ed25519 private key"))?;
            ed25519_dalek::SigningKey::from_bytes(arr);
            Ok(())
        }
        SignatureScheme::Secp256k1 => {
            k256::ecdsa::SigningKey::from_slice(bytes).map_err(|_| SignerError::invalid_input("Invalid secp256k1 private key"))?;
            Ok(())
        }
    }
}

fn decode_key(value: &str, encodings: &[KeyEncoding], scheme: SignatureScheme) -> Result<Zeroizing<Vec<u8>>, SignerError> {
    for encoding in encodings {
        let decoded = match encoding {
            KeyEncoding::Hex => decode_hex(value).ok(),
            KeyEncoding::Base58 => decode_base58(value),
            KeyEncoding::Base32 => decode_base32_stellar(value),
        };
        if let Some(bytes) = decoded
            && validate_key(&bytes, scheme).is_ok()
        {
            return Ok(Zeroizing::new(bytes));
        }
    }
    Err(SignerError::invalid_input("Invalid private key format"))
}

fn encode_key(bytes: &[u8], encoding: KeyEncoding) -> Result<String, SignerError> {
    match encoding {
        KeyEncoding::Hex => Ok(encode_with_0x(bytes)),
        KeyEncoding::Base58 => Ok(bs58::encode(bytes).into_string()),
        KeyEncoding::Base32 => Err(SignerError::invalid_input("Unsupported private key export encoding")),
    }
}

pub fn decode_private_key(chain: &Chain, value: &str) -> Result<Zeroizing<Vec<u8>>, SignerError> {
    let import_encodings = import_encodings_for_chain(chain);
    let scheme = scheme_for_chain(chain);
    decode_key(value, import_encodings, scheme)
}

pub fn encode_private_key(chain: &Chain, private_key: &[u8]) -> Result<String, SignerError> {
    encode_key(private_key, export_encoding_for_chain(chain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_solana_base58() {
        let bytes = decode_private_key(&Chain::Solana, "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC").unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_decode_ethereum_hex() {
        let bytes = decode_private_key(&Chain::Ethereum, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_decode_stellar_strkey() {
        let bytes = decode_private_key(&Chain::Stellar, "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ").unwrap();
        assert_eq!(hex::encode(bytes.as_slice()), "3d769e8a65b9002a470e9aecf2587ef848e2a0b483320e24c493a5913d594eb9");
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_private_key(&Chain::Ethereum, "not_valid").is_err());
        assert!(decode_private_key(&Chain::Stellar, "GA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRJ").is_err());
    }

    #[test]
    fn test_encode_solana_base58() {
        let bytes = decode_private_key(&Chain::Solana, "4ha2npeRkDXipjgGJ3L5LhZ9TK9dRjP2yktydkFBhAzXj3N8ytpYyTS24kxcYGEefy4WKWRcog2zSPvpPZoGmxCC").unwrap();
        let encoded = encode_private_key(&Chain::Solana, &bytes).unwrap();

        assert_eq!(encoded, "DTJi5pMtSKZHdkLX4wxwvjGjf2xwXx1LSuuUZhugYWDV");
    }

    #[test]
    fn test_encode_ethereum_hex() {
        let bytes = decode_private_key(&Chain::Ethereum, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        let encoded = encode_private_key(&Chain::Ethereum, &bytes).unwrap();

        assert_eq!(encoded, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e");
    }

    #[test]
    fn test_encode_bitcoin_base58() {
        let bytes = decode_private_key(&Chain::Ethereum, "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e").unwrap();
        let encoded = encode_private_key(&Chain::Bitcoin, &bytes).unwrap();

        assert_eq!(encoded, "4Hmr8TxnwVB7m6fzfPRcASMn2hLRgzUz3gDGwF4ZnpVK");
    }

    #[test]
    fn test_encode_does_not_revalidate_bytes() {
        assert_eq!(encode_private_key(&Chain::Ethereum, &[1u8; 16]).unwrap(), "0x01010101010101010101010101010101");
        assert_eq!(encode_private_key(&Chain::Solana, &[1u8; 64]).unwrap(), bs58::encode([1u8; 64]).into_string());
    }

    #[test]
    fn test_stellar_bad_checksum() {
        // Valid stellar key with last char changed to corrupt checksum
        assert!(decode_private_key(&Chain::Stellar, "SA6XNHUKMW4QAKSHB2NOZ4SYP34ERYVAWSBTEDREYSJ2LEJ5LFHLTIRA").is_err());
    }

    #[test]
    fn test_supports_private_key_import() {
        assert!(supports_private_key_import(&Chain::Ethereum));
        assert!(supports_private_key_import(&Chain::Solana));
        assert!(supports_private_key_import(&Chain::Stellar));
        assert!(!supports_private_key_import(&Chain::Bitcoin));
        assert!(!supports_private_key_import(&Chain::Litecoin));
        assert!(!supports_private_key_import(&Chain::Doge));
    }

    #[test]
    fn test_base58_rejects_unexpected_length() {
        // 96-byte base58 payload should not be accepted
        let long_key = bs58::encode(vec![1u8; 96]).into_string();
        assert!(decode_base58(&long_key).is_none());
    }
}
