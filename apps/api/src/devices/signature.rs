use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::{DeviceAuthPayload, decode_signature, parse_device_auth, verify_device_signature};
use rocket::Request;
use rocket::http::Status;

use crate::devices::constants::{AUTHORIZATION_HEADER, HEADER_DEVICE_BODY_HASH, HEADER_DEVICE_ID, HEADER_DEVICE_SIGNATURE, HEADER_DEVICE_TIMESTAMP};
use crate::devices::error::DeviceError;

pub fn parse_auth_components(req: &Request<'_>) -> Result<DeviceAuthPayload, DeviceError> {
    if let Some(auth_value) = req.headers().get_one(AUTHORIZATION_HEADER)
        && auth_value.starts_with(gem_auth::GEM_AUTH_SCHEME)
    {
        return parse_device_auth(auth_value).ok_or(DeviceError::InvalidAuthorizationFormat);
    }

    let device_id = req.headers().get_one(HEADER_DEVICE_ID).ok_or(DeviceError::MissingHeader(HEADER_DEVICE_ID))?;
    let timestamp = req.headers().get_one(HEADER_DEVICE_TIMESTAMP).ok_or(DeviceError::MissingHeader(HEADER_DEVICE_TIMESTAMP))?;
    let body_hash = req.headers().get_one(HEADER_DEVICE_BODY_HASH).ok_or(DeviceError::MissingHeader(HEADER_DEVICE_BODY_HASH))?;
    let signature = req.headers().get_one(HEADER_DEVICE_SIGNATURE).ok_or(DeviceError::MissingHeader(HEADER_DEVICE_SIGNATURE))?;
    let signature = decode_signature(signature).ok_or(DeviceError::InvalidSignature)?;

    Ok(DeviceAuthPayload {
        scheme: gem_auth::AuthScheme::Legacy,
        device_id: device_id.to_string(),
        timestamp: timestamp.to_string(),
        wallet_id: None,
        body_hash: body_hash.to_string(),
        signature,
    })
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
    let message = match components.scheme {
        gem_auth::AuthScheme::Gem => {
            let wallet_id = components.wallet_id.as_deref().unwrap_or("");
            format!("{}.{}.{}.{}.{}", components.timestamp, method, path, wallet_id, components.body_hash)
        }
        gem_auth::AuthScheme::Legacy => {
            format!("v1.{}.{}.{}.{}", components.timestamp, method, path, components.body_hash)
        }
    };

    if !verify_device_signature(&components.device_id, &message, &components.signature) {
        return Err((Status::Unauthorized, DeviceError::InvalidSignature.to_string()));
    }

    Ok(())
}
