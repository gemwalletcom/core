use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::Outcome;
use storage::database::devices::DevicesStore;
use storage::models::{DeviceRow, WalletRow};
use storage::{Database, DatabaseClient, WalletsRepository};

use crate::devices::auth_config::AuthConfig;
use crate::devices::constants::DEVICE_ID_LENGTH;
use crate::devices::error::DeviceError;
use crate::devices::signature::{parse_auth_components, verify_request_signature};
use crate::responders::cache_error;

pub(super) struct AuthResult {
    pub(super) device_id: String,
    pub(super) wallet_id: Option<String>,
}

pub(super) fn auth_error_outcome<T>(req: &Request<'_>, error: DeviceError, device_id: Option<&str>, wallet_id: Option<&str>) -> Outcome<T, String> {
    let status = match error {
        DeviceError::MissingHeader(_)
        | DeviceError::InvalidDeviceId
        | DeviceError::InvalidTimestamp
        | DeviceError::TimestampExpired
        | DeviceError::InvalidSignature
        | DeviceError::MissingWalletId
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

pub(super) async fn authenticate<T>(req: &Request<'_>) -> Result<AuthResult, Outcome<T, String>> {
    let Success(config) = req.guard::<&rocket::State<AuthConfig>>().await else {
        panic!("AuthConfig not configured");
    };

    let components = parse_auth_components(req).map_err(|e| auth_error_outcome(req, e, None, None))?;

    if components.device_id.len() != DEVICE_ID_LENGTH {
        return Err(auth_error_outcome(req, DeviceError::InvalidDeviceId, Some(&components.device_id), None));
    }

    verify_request_signature(req, &components, config.tolerance.as_millis() as u64).map_err(|(status, msg)| {
        cache_error(req, &msg);
        Error((status, msg))
    })?;

    Ok(AuthResult {
        device_id: components.device_id,
        wallet_id: components.wallet_id,
    })
}

pub(super) async fn lookup_device<T>(req: &Request<'_>, device_id: &str) -> Result<(DeviceRow, DatabaseClient), Outcome<T, String>> {
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

pub(super) async fn lookup_device_wallet<T>(req: &Request<'_>, device_id: &str, wallet_id: &str) -> Result<(DeviceRow, WalletRow), Outcome<T, String>> {
    let (device_row, mut db_client) = lookup_device(req, device_id).await?;

    let wallet_row = match db_client.get_wallet_by_device_and_identifier(device_row.id, wallet_id) {
        Ok(wallet_row) => wallet_row,
        Err(error) if error.is_not_found() => return Err(auth_error_outcome(req, DeviceError::WalletNotFound, Some(device_id), Some(wallet_id))),
        Err(_) => return Err(auth_error_outcome(req, DeviceError::DatabaseError, Some(device_id), Some(wallet_id))),
    };

    Ok((device_row, wallet_row))
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
