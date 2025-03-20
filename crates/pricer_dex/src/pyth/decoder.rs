use base64::{engine::general_purpose, Engine as _};

use super::model::Price;

pub fn decode_price_account(base64_str: &str) -> Result<Price, Box<dyn std::error::Error + Send + Sync>> {
    let data = general_purpose::STANDARD.decode(base64_str)?;

    let price_exponent = i32::from_le_bytes(data[20..24].try_into()?);
    println!("🔹 Price Exponent: {:?}", price_exponent);

    let price = i64::from_le_bytes(data[208..216].try_into()?);
    println!("🔹 Price: {}", price);

    let scaled_price = price as f64 * 10f64.powi(price_exponent);
    println!("✅  Scaled Price: ${:.2}", scaled_price);

    // let magic = u32::from_le_bytes(account_data[0..4].try_into()?);
    //     println!("🔹 magic: {:?}", magic);

    //     let version = u32::from_le_bytes(account_data[4..8].try_into()?);
    //     println!("🔹 version: {:?}", version);

    //     let account_type = u32::from_le_bytes(account_data[8..12].try_into()?);
    //     println!("🔹 account_type: {:?}", account_type);

    //     let account_type = u32::from_le_bytes(account_data[12..16].try_into()?);
    //     println!("🔹 size: {:?}", account_type);

    //     let price_type = u32::from_le_bytes(account_data[16..20].try_into()?);
    //     println!("🔹 price_type: {:?}", price_type);

    //     let price_exponent = i32::from_le_bytes(account_data[20..24].try_into()?);
    //     println!("🔹 Price Exponent: {:?}", price_exponent);

    //     let last_slot = i64::from_le_bytes(account_data[32..40].try_into()?);
    //     println!("🔹 Last slot: {}", last_slot);

    //     println!("🔹 Account data length: {}", account_data.len());

    //     let price = i64::from_le_bytes(account_data[208..216].try_into()?);
    //     println!("🔹 Price: {}", price);

    //     let scaled_price = price as f64 * 10f64.powi(price_exponent);
    //     println!("✅  Scaled Price: ${:.2}", scaled_price);

    Ok(Price { price: scaled_price })
}
