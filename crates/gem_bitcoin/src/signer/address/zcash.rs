use primitives::SignerError;

use super::script::{AddressScript, LockingScript, p2pkh_script, p2sh_script};
use crate::hash::hash20;

// Zcash mainnet transparent address version bytes: t1 for P2PKH, t3 for P2SH.
pub(crate) const TRANSPARENT_P2PKH_PREFIX: [u8; 2] = [0x1c, 0xb8];
pub(crate) const TRANSPARENT_P2SH_PREFIX: [u8; 2] = [0x1c, 0xbd];

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    let payload = bs58::decode(address).with_check(None).into_vec().map_err(SignerError::from_display)?;
    if payload.len() != 22 {
        return Err(SignerError::from_display("invalid Zcash address"));
    }
    let hash = hash20(&payload[2..])?;
    match [payload[0], payload[1]] {
        TRANSPARENT_P2PKH_PREFIX => Ok(AddressScript::new(p2pkh_script(hash), LockingScript::P2pkh)),
        TRANSPARENT_P2SH_PREFIX => Ok(AddressScript::new(p2sh_script(hash), LockingScript::P2sh)),
        _ => Err(SignerError::from_display("unsupported Zcash address version")),
    }
}
