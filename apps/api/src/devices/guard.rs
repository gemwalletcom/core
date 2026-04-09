use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use crate::devices::auth_config::AuthConfig;
use crate::devices::constants::{DEVICE_ID_LENGTH, HEADER_DEVICE_ID, HEADER_WALLET_ID};
use crate::devices::error::DeviceError;
use crate::devices::signature::{parse_auth_components, verify_request_signature};
use crate::responders::cache_error;

fn auth_error_outcome<T>(req: &Request<'_>, error: DeviceError, device_id: Option<&str>, wallet_id: Option<&str>) -> Outcome<T, String> {
    let status = match error {
        DeviceError::MissingHeader(_)
        | DeviceError::InvalidDeviceId
        | DeviceError::InvalidTimestamp
        | DeviceError::TimestampExpired
        | DeviceError::InvalidSignature
        | DeviceError::InvalidAuthorizationFormat => Status::Unauthorized,
        DeviceError::DeviceNotFound | DeviceError::WalletNotFound => Status::NotFound,
        DeviceError::DatabaseUnavailable | DeviceError::DatabaseError => Status::InternalServerError,
    };
    let message = format_auth_error_message(&error, device_id, wallet_id);
    cache_error(req, &message);
    Error((status, message))
}

fn format_auth_error_message(error: &DeviceError, device_id: Option<&str>, wallet_id: Option<&str>) -> String {
    let mut message = error.to_string();
    if let Some(id) = device_id {
        message.push_str(&format!(" device_id={id}"));
    }
    if let Some(id) = wallet_id {
        message.push_str(&format!(" wallet_id={id}"));
    }
    message
}

struct AuthResult {
    device_id: String,
    wallet_id: Option<String>,
}

async fn authenticate<T>(req: &Request<'_>) -> Result<AuthResult, Outcome<T, String>> {
    let Success(config) = req.guard::<&rocket::State<AuthConfig>>().await else {
        panic!("AuthConfig not configured");
    };

    if !config.enabled {
        let device_id = req
            .headers()
            .get_one(HEADER_DEVICE_ID)
            .ok_or_else(|| auth_error_outcome(req, DeviceError::MissingHeader(HEADER_DEVICE_ID), None, None))?;

        if device_id.len() != DEVICE_ID_LENGTH {
            return Err(auth_error_outcome(req, DeviceError::InvalidDeviceId, Some(device_id), None));
        }

        return Ok(AuthResult {
            device_id: device_id.to_string(),
            wallet_id: req.headers().get_one(HEADER_WALLET_ID).map(|s| s.to_string()),
        });
    }

    let components = parse_auth_components(req).map_err(|e| auth_error_outcome(req, e, None, None))?;

    if components.device_id.len() != DEVICE_ID_LENGTH {
        return Err(auth_error_outcome(req, DeviceError::InvalidDeviceId, Some(&components.device_id), None));
    }

    verify_request_signature(req, &components, config.tolerance.as_millis() as u64).map_err(|(status, msg)| {
        cache_error(req, &msg);
        Error((status, msg))
    })?;

    let wallet_id = components.wallet_id.clone().or_else(|| req.headers().get_one(HEADER_WALLET_ID).map(|s| s.to_string()));

    Ok(AuthResult {
        device_id: components.device_id,
        wallet_id,
    })
}

// Signature verified, no database check (for device registration)
pub struct VerifiedDeviceId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VerifiedDeviceId {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        match authenticate(req).await {
            Ok(auth) => Success(VerifiedDeviceId(auth.device_id)),
            Err(error) => error,
        }
    }
}

// Signature verified + device exists in database
pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let auth = match authenticate(req).await {
            Ok(auth) => auth,
            Err(error) => return error,
        };

        let (device_row, _) = match lookup_device(req, &auth.device_id).await {
            Ok(result) => result,
            Err(error) => return error,
        };

        Success(AuthenticatedDevice { device_row })
    }
}

// Signature verified + device and wallet exist in database
pub struct AuthenticatedDeviceWallet {
    pub device_row: DeviceRow,
    pub wallet_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDeviceWallet {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let auth = match authenticate(req).await {
            Ok(auth) => auth,
            Err(error) => return error,
        };

        let Some(wallet_id_str) = auth.wallet_id else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_WALLET_ID), Some(&auth.device_id), None);
        };

        let (device_row, mut db_client) = match lookup_device(req, &auth.device_id).await {
            Ok(result) => result,
            Err(error) => return error,
        };

        let Ok(wallet_row) = WalletsStore::get_wallet(&mut db_client, &wallet_id_str) else {
            return auth_error_outcome(req, DeviceError::WalletNotFound, Some(&auth.device_id), Some(&wallet_id_str));
        };

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
        })
    }
}

async fn lookup_device<T>(req: &Request<'_>, device_id: &str) -> Result<(DeviceRow, storage::DatabaseClient), Outcome<T, String>> {
    let Success(database) = req.guard::<&rocket::State<Database>>().await else {
        return Err(auth_error_outcome(req, DeviceError::DatabaseUnavailable, Some(device_id), None));
    };

    let Ok(mut db_client) = database.client() else {
        return Err(auth_error_outcome(req, DeviceError::DatabaseError, Some(device_id), None));
    };

    let Ok(device_row) = DevicesStore::get_device(&mut db_client, device_id) else {
        return Err(auth_error_outcome(req, DeviceError::DeviceNotFound, Some(device_id), None));
    };

    Ok((device_row, db_client))
}

#[cfg(test)]
mod tests {
    use super::format_auth_error_message;
    use crate::devices::error::DeviceError;

    #[test]
    fn test_format_auth_error_message_includes_wallet_id() {
        let message = format_auth_error_message(&DeviceError::WalletNotFound, Some("device_123"), Some("wallet_456"));

        assert_eq!(message, "Wallet not found device_id=device_123 wallet_id=wallet_456");
    }
}
