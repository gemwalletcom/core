use crate::GemstoneError;
use gem_cosmos::converter::convert_cosmos_address;

/// Exports functions
#[uniffi::export]
pub fn cosmos_convert_hrp(address: String, hrp: String) -> Result<String, GemstoneError> {
    convert_cosmos_address(&address, &hrp).map_err(GemstoneError::from)
}
