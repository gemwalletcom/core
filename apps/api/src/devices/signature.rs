use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::verify_device_signature;
use rocket::Request;
use rocket::http::Status;

use crate::devices::constants::{HEADER_DEVICE_BODY_HASH, HEADER_DEVICE_SIGNATURE, HEADER_DEVICE_TIMESTAMP, SIGNATURE_TIMESTAMP_TOLERANCE_MS};
use crate::devices::error::DeviceError;

pub fn verify_request_signature(req: &Request<'_>, public_key: &str) -> Result<(), (Status, String)> {
    let signature = req
        .headers()
        .get_one(HEADER_DEVICE_SIGNATURE)
        .ok_or_else(|| (Status::Unauthorized, DeviceError::MissingHeader(HEADER_DEVICE_SIGNATURE).to_string()))?;
    let timestamp_str = req
        .headers()
        .get_one(HEADER_DEVICE_TIMESTAMP)
        .ok_or_else(|| (Status::Unauthorized, DeviceError::MissingHeader(HEADER_DEVICE_TIMESTAMP).to_string()))?;
    let body_hash = req
        .headers()
        .get_one(HEADER_DEVICE_BODY_HASH)
        .ok_or_else(|| (Status::Unauthorized, DeviceError::MissingHeader(HEADER_DEVICE_BODY_HASH).to_string()))?;

    let timestamp_ms: u64 = timestamp_str.parse().map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?
        .as_millis() as u64;

    if now_ms.abs_diff(timestamp_ms) > SIGNATURE_TIMESTAMP_TOLERANCE_MS {
        return Err((Status::Unauthorized, DeviceError::TimestampExpired.to_string()));
    }

    let method = req.method().as_str();
    let path = req.uri().path().as_str();
    let message = format!("v1.{timestamp_str}.{method}.{path}.{body_hash}");

    if !verify_device_signature(public_key, &message, signature) {
        return Err((Status::Unauthorized, DeviceError::InvalidSignature.to_string()));
    }

    Ok(())
}
