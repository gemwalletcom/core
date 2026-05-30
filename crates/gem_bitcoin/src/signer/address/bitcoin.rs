use std::str::FromStr;

use ::bitcoin::{Address, AddressType, Network, address::AddressData};
use primitives::SignerError;

use super::script::{AddressScript, LockingScript};

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    let address = Address::from_str(address)
        .map_err(SignerError::from_display)?
        .require_network(Network::Bitcoin)
        .map_err(SignerError::from_display)?;
    let script_pubkey = address.script_pubkey();

    match address.to_address_data() {
        AddressData::P2pkh { .. } => Ok(AddressScript::new(script_pubkey, LockingScript::P2pkh)),
        AddressData::P2sh { .. } => Ok(AddressScript::new(script_pubkey, LockingScript::P2sh)),
        AddressData::Segwit { .. } => match address.address_type() {
            Some(AddressType::P2wpkh) => Ok(AddressScript::new(script_pubkey, LockingScript::P2wpkh)),
            Some(AddressType::P2wsh) => Ok(AddressScript::new(script_pubkey, LockingScript::P2wsh)),
            Some(AddressType::P2tr) => Ok(AddressScript::new(script_pubkey, LockingScript::P2tr)),
            _ => Err(SignerError::from_display("unsupported Bitcoin address type")),
        },
        _ => Err(SignerError::from_display("unsupported Bitcoin address type")),
    }
}
