use crate::address::StellarAddress;
use primitives::{Address, SignerError, SignerInput};
use signer::InvalidInput;

#[derive(Clone)]
pub enum StellarAssetCode {
    Alphanum4([u8; 4]),
    Alphanum12([u8; 12]),
}

#[derive(Clone)]
pub struct StellarAssetData {
    pub issuer: StellarAddress,
    pub code: StellarAssetCode,
}

impl StellarAssetData {
    pub fn new(issuer: &str, code: &str) -> Result<Self, SignerError> {
        if !(code.is_ascii() && code.bytes().all(|byte| byte.is_ascii_alphanumeric())) {
            return SignerError::invalid_input_err("Stellar asset code must be ASCII alphanumeric");
        }

        let code = match code.len() {
            1..=4 => {
                let mut buf = [0u8; 4];
                buf[..code.len()].copy_from_slice(code.as_bytes());
                StellarAssetCode::Alphanum4(buf)
            }
            5..=12 => {
                let mut buf = [0u8; 12];
                buf[..code.len()].copy_from_slice(code.as_bytes());
                StellarAssetCode::Alphanum12(buf)
            }
            _ => return Err(SignerError::invalid_input("Stellar asset code must be 1-12 characters")),
        };

        let issuer = StellarAddress::from_str(issuer).invalid_input("invalid Stellar issuer address")?;
        Ok(Self { issuer, code })
    }

    pub(crate) fn from_input(input: &SignerInput) -> Result<Self, SignerError> {
        let (issuer, code) = input.input_type.get_asset().id.split_sub_token_parts().invalid_input("invalid Stellar token ID")?;
        Self::new(&issuer, &code)
    }
}
