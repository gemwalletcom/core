use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::device_sessions::DeviceSessionsStore;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use gem_auth::DeviceAuthPayload;

use crate::devices::auth_config::AuthConfig;
use crate::devices::constants::{DEVICE_ID_LENGTH, HEADER_WALLET_ID};
use crate::devices::error::DeviceError;
use crate::devices::signature::{parse_auth_components, verify_request_signature};
use crate::responders::cache_error;

fn auth_error_outcome<T>(req: &Request<'_>, error: DeviceError, device_id: Option<&str>) -> Outcome<T, String> {
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
    let message = match device_id {
        Some(id) => format!("{} device_id={}", error, id),
        None => error.to_string(),
    };
    cache_error(req, &message);
    Error((status, message))
}

fn get_validated_auth<T>(req: &Request<'_>) -> Result<DeviceAuthPayload, Outcome<T, String>> {
    let components = parse_auth_components(req).map_err(|e| auth_error_outcome(req, e, None))?;

    if components.device_id.len() != DEVICE_ID_LENGTH {
        return Err(auth_error_outcome(req, DeviceError::InvalidDeviceId, Some(&components.device_id)));
    }

    Ok(components)
}

async fn verify_signature(req: &Request<'_>, components: &DeviceAuthPayload) -> Result<(), (Status, String)> {
    let Success(config) = req.guard::<&rocket::State<AuthConfig>>().await else {
        panic!("AuthConfig not configured");
    };
    if !config.enabled {
        return Ok(());
    }
    verify_request_signature(req, components, config.tolerance.as_millis() as u64)
}

// Signature verified + device exists in database
pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let components = match get_validated_auth(req) {
            Ok(c) => c,
            Err(error) => return error,
        };

        if let Err((status, msg)) = verify_signature(req, &components).await {
            cache_error(req, &msg);
            return Error((status, msg));
        }

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable, Some(&components.device_id));
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError, Some(&components.device_id));
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &components.device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound, Some(&components.device_id));
        };

        if DeviceSessionsStore::add_device_session(&mut db_client, device_row.id).is_err() {
            return auth_error_outcome(req, DeviceError::DatabaseError, Some(&components.device_id));
        }

        Success(AuthenticatedDevice { device_row })
    }
}

// Signature verified, no database check (for device registration)
pub struct VerifiedDeviceId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for VerifiedDeviceId {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        match get_validated_auth(req) {
            Ok(components) => {
                if let Err((status, msg)) = verify_signature(req, &components).await {
                    cache_error(req, &msg);
                    return Error((status, msg));
                }
                Success(VerifiedDeviceId(components.device_id))
            }
            Err(error) => error,
        }
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
        let components = match get_validated_auth(req) {
            Ok(c) => c,
            Err(error) => return error,
        };

        let wallet_id_str = components.wallet_id.as_deref().or_else(|| req.headers().get_one(HEADER_WALLET_ID));

        let Some(wallet_id_str) = wallet_id_str else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_WALLET_ID), Some(&components.device_id));
        };

        if let Err((status, msg)) = verify_signature(req, &components).await {
            cache_error(req, &msg);
            return Error((status, msg));
        }

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable, Some(&components.device_id));
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError, Some(&components.device_id));
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &components.device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound, Some(&components.device_id));
        };

        let Ok(wallet_row) = WalletsStore::get_wallet(&mut db_client, wallet_id_str) else {
            return auth_error_outcome(req, DeviceError::WalletNotFound, Some(&components.device_id));
        };

        if DeviceSessionsStore::add_device_session(&mut db_client, device_row.id).is_err() {
            return auth_error_outcome(req, DeviceError::DatabaseError, Some(&components.device_id));
        }

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
        })
    }
}
