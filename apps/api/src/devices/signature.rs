use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::{DeviceAuthPayload, parse_device_auth, verify_device_signature};
use rocket::Request;
use rocket::http::Status;

use crate::devices::constants::AUTHORIZATION_HEADER;
use crate::devices::error::DeviceError;

pub fn parse_auth_components(req: &Request<'_>) -> Result<DeviceAuthPayload, DeviceError> {
    let auth_value = req.headers().get_one(AUTHORIZATION_HEADER).ok_or(DeviceError::MissingHeader(AUTHORIZATION_HEADER))?;
    parse_device_auth(auth_value).ok_or(DeviceError::InvalidAuthorizationFormat)
}

pub fn verify_request_signature(req: &Request<'_>, components: &DeviceAuthPayload, tolerance_ms: u64) -> Result<(), (Status, String)> {
    let timestamp_ms: u64 = components
        .timestamp
        .parse()
        .map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?
        .as_millis() as u64;

    if now_ms.abs_diff(timestamp_ms) > tolerance_ms {
        return Err((Status::Unauthorized, DeviceError::TimestampExpired.to_string()));
    }

    let method = req.method().as_str();
    let path = req.uri().path().as_str();
    let wallet_id = components.wallet_id.as_deref().unwrap_or("");
    let message = format!("{}.{}.{}.{}.{}", components.timestamp, method, path, wallet_id, components.body_hash);

    if !verify_device_signature(&components.device_id, &message, &components.signature) {
        return Err((Status::Unauthorized, DeviceError::InvalidSignature.to_string()));
    }

    Ok(())
}
