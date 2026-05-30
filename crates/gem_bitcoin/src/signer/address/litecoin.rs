use primitives::SignerError;

use super::script::AddressScript;

const P2PKH_VERSIONS: [u8; 1] = [48];
const P2SH_VERSIONS: [u8; 2] = [5, 50];
const HRP: &str = "ltc";

pub(super) fn script(address: &str) -> Result<AddressScript, SignerError> {
    AddressScript::from_prefixed_address(address, &P2PKH_VERSIONS, &P2SH_VERSIONS, Some(HRP))
}
