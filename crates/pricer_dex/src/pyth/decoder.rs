use base64::{engine::general_purpose, Engine as _};

use super::model::Price;

// https://github.com/pyth-network/pyth-sdk-rs/blob/main/pyth-sdk-solana/src/state.rs#L288
pub fn decode_price_account(base64_str: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
    let data = general_purpose::STANDARD.decode(base64_str)?;
    let price_exponent = i32::from_le_bytes(data[20..24].try_into()?);
    let price = i64::from_le_bytes(data[208..216].try_into()?);

    // let last_slot = i64::from_le_bytes(account_data[32..40].try_into()?);
    // println!("🔹 Last slot: {}", last_slot);

    let scaled_price = price as f64 * 10f64.powi(price_exponent);

    Ok(Price { price: scaled_price })
}
