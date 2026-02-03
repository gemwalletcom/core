use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::device_sessions::DeviceSessionsStore;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use crate::devices::auth_config::AuthConfig;
use crate::devices::constants::{DEVICE_ID_LENGTH, HEADER_DEVICE_ID, HEADER_WALLET_ID};
use crate::devices::error::DeviceError;
use crate::devices::signature::verify_request_signature;
use crate::responders::cache_error;

fn auth_error_outcome<T>(req: &Request<'_>, error: DeviceError) -> Outcome<T, String> {
    let status = match error {
        DeviceError::MissingHeader(_) | DeviceError::InvalidDeviceId | DeviceError::InvalidTimestamp | DeviceError::TimestampExpired | DeviceError::InvalidSignature => {
            Status::Unauthorized
        }
        DeviceError::DeviceNotFound | DeviceError::WalletNotFound => Status::NotFound,
        DeviceError::DatabaseUnavailable | DeviceError::DatabaseError => Status::InternalServerError,
    };
    let message = error.to_string();
    cache_error(req, &message);
    Error((status, message))
}

fn get_validated_device_id<T>(req: &Request<'_>) -> Result<String, Outcome<T, String>> {
    let device_id = req
        .headers()
        .get_one(HEADER_DEVICE_ID)
        .ok_or_else(|| auth_error_outcome(req, DeviceError::MissingHeader(HEADER_DEVICE_ID)))?;

    if device_id.len() != DEVICE_ID_LENGTH {
        return Err(auth_error_outcome(req, DeviceError::InvalidDeviceId));
    }

    Ok(device_id.to_string())
}

async fn verify_signature(req: &Request<'_>, device_id: &str) -> Result<(), (Status, String)> {
    let Success(config) = req.guard::<&rocket::State<AuthConfig>>().await else {
        panic!("AuthConfig not configured");
    };
    if !config.enabled {
        return Ok(());
    }
    verify_request_signature(req, device_id, config.tolerance.as_millis() as u64)
}

// Signature verified + device exists in database
pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let device_id = match get_validated_device_id(req) {
            Ok(id) => id,
            Err(error) => return error,
        };

        if let Err((status, msg)) = verify_signature(req, &device_id).await {
            cache_error(req, &msg);
            return Error((status, msg));
        }

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable);
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError);
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound);
        };

        if DeviceSessionsStore::add_device_session(&mut db_client, device_row.id).is_err() {
            return auth_error_outcome(req, DeviceError::DatabaseError);
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
        match get_validated_device_id(req) {
            Ok(id) => {
                if let Err((status, msg)) = verify_signature(req, &id).await {
                    cache_error(req, &msg);
                    return Error((status, msg));
                }
                Success(VerifiedDeviceId(id))
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
        let device_id = match get_validated_device_id(req) {
            Ok(id) => id,
            Err(error) => return error,
        };

        let Some(wallet_id_str) = req.headers().get_one(HEADER_WALLET_ID) else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_WALLET_ID));
        };

        if let Err((status, msg)) = verify_signature(req, &device_id).await {
            cache_error(req, &msg);
            return Error((status, msg));
        }

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable);
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError);
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound);
        };

        let Ok(wallet_row) = WalletsStore::get_wallet(&mut db_client, wallet_id_str) else {
            return auth_error_outcome(req, DeviceError::WalletNotFound);
        };

        if DeviceSessionsStore::add_device_session(&mut db_client, device_row.id).is_err() {
            return auth_error_outcome(req, DeviceError::DatabaseError);
        }

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
        })
    }
}
