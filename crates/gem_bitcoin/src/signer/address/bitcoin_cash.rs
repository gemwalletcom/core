use std::fmt;

use bitcoincash_addr::{
    Address as CashAddress, HashType as CashHashType, Network as CashNetwork, base58::DecodingError as Base58DecodingError, cashaddr::DecodingError as CashaddrDecodingError,
};
use primitives::SignerError;

use super::script::{AddressScript, LockingScript, p2pkh_script, p2sh_script};
use crate::hash::hash20;

const CASHADDR_PREFIX: &str = "bitcoincash:";

struct DecodeAddressError {
    cashaddr: CashaddrDecodingError,
    base58: Base58DecodingError,
}

impl From<(CashaddrDecodingError, Base58DecodingError)> for DecodeAddressError {
    fn from((cashaddr, base58): (CashaddrDecodingError, Base58DecodingError)) -> Self {
        Self { cashaddr, base58 }
    }
}

impl fmt::Display for DecodeAddressError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid Bitcoin Cash address: cashaddr: {}; base58: {}", self.cashaddr, self.base58)
    }
}

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    let address = decode_address(address).map_err(SignerError::from_display)?;
    if address.network != CashNetwork::Main {
        return Err(SignerError::from_display("unsupported Bitcoin Cash address network"));
    }
    let hash = hash20(&address.body)?;
    match address.hash_type {
        CashHashType::Key => Ok(AddressScript::new(p2pkh_script(hash), LockingScript::P2pkh)),
        CashHashType::Script => Ok(AddressScript::new(p2sh_script(hash), LockingScript::P2sh)),
    }
}

fn decode_address(address: &str) -> Result<CashAddress, DecodeAddressError> {
    match CashAddress::decode(address) {
        Ok(address) => Ok(address),
        Err(error) => {
            if address.contains(':') {
                return Err(error.into());
            }
            CashAddress::decode(&format!("{CASHADDR_PREFIX}{address}")).map_err(|_| error.into())
        }
    }
}
