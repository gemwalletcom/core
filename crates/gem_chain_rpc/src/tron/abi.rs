use alloy_primitives::U256;
use std::error::Error;

use alloy_sol_types::SolValue;

pub fn decode_abi_string(hex_data: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let bytes_data = hex::decode(hex_data).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    if bytes_data.is_empty() {
        return Ok("".to_string());
    }
    // Try to decode as ABI string. If that fails, try to interpret as a direct UTF-8 string.
    String::abi_decode(&bytes_data).or_else(|_abi_error| {
        String::from_utf8(bytes_data)
            .map(|s| s.trim_matches('\0').to_string())
            .map_err(|utf8_error| Box::new(utf8_error) as Box<dyn Error + Send + Sync>)
    })
}

pub fn decode_abi_uint8(hex_data: &str) -> Result<u8, Box<dyn Error + Send + Sync>> {
    if hex_data.is_empty() {
        return Ok(0);
    }
    let bytes_data = hex::decode(hex_data).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    let value_u256 = U256::abi_decode(&bytes_data).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;
    value_u256.try_into().map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
}
