use gem_hash::sha2::sha512_256;
use primitives::SignerError;
use signer::decode_base32;

pub(crate) use signer::Base32Address;

const ADDRESS_LENGTH: usize = 58;
const ADDRESS_DATA_LENGTH: usize = 32;
const ADDRESS_CHECKSUM_LENGTH: usize = 4;
const INVALID_ADDRESS: &str = "invalid Algorand address";

pub(crate) fn parse_address(value: &str) -> Result<Base32Address, SignerError> {
    let decoded = (|| -> Result<Vec<u8>, &'static str> {
        if value.len() != ADDRESS_LENGTH {
            return Err(INVALID_ADDRESS);
        }

        let decoded = decode_base32(value.as_bytes()).ok_or(INVALID_ADDRESS)?;
        if decoded.len() != ADDRESS_DATA_LENGTH + ADDRESS_CHECKSUM_LENGTH {
            return Err(INVALID_ADDRESS);
        }
        Ok(decoded)
    })()
    .map_err(SignerError::from_display)?;

    let address = Base32Address::from_slice(&decoded[..ADDRESS_DATA_LENGTH])?;
    let checksum = address_checksum(address.payload());
    if decoded[ADDRESS_DATA_LENGTH..] != checksum {
        return Err(SignerError::invalid_input("invalid Algorand address checksum"));
    }

    Ok(address)
}

fn address_checksum(bytes: &[u8; ADDRESS_DATA_LENGTH]) -> [u8; ADDRESS_CHECKSUM_LENGTH] {
    let digest = sha512_256(bytes);
    let mut checksum = [0u8; ADDRESS_CHECKSUM_LENGTH];
    checksum.copy_from_slice(&digest[digest.len() - ADDRESS_CHECKSUM_LENGTH..]);
    checksum
}
