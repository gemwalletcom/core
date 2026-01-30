use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::verify_device_signature;
use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use crate::responders::cache_error;

const TIMESTAMP_TOLERANCE_MS: u64 = 300_000;

fn error_outcome<T>(req: &Request<'_>, status: Status, message: &str) -> Outcome<T, String> {
    cache_error(req, message);
    Error((status, message.to_string()))
}

fn verify_signature(req: &Request<'_>, device_row: &DeviceRow) -> Result<(), (Status, String)> {
    let Some(ref public_key) = device_row.public_key else {
        return Ok(());
    };

    let signature = req
        .headers()
        .get_one("x-device-signature")
        .ok_or((Status::Unauthorized, "Missing x-device-signature".to_string()))?;
    let timestamp_str = req
        .headers()
        .get_one("x-device-timestamp")
        .ok_or((Status::Unauthorized, "Missing x-device-timestamp".to_string()))?;
    let body_hash = req
        .headers()
        .get_one("x-device-body-hash")
        .ok_or((Status::Unauthorized, "Missing x-device-body-hash".to_string()))?;

    let timestamp_ms: u64 = timestamp_str.parse().map_err(|_| (Status::Unauthorized, "Invalid timestamp".to_string()))?;

    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;

    if now_ms.abs_diff(timestamp_ms) > TIMESTAMP_TOLERANCE_MS {
        return Err((Status::Unauthorized, "Timestamp expired".to_string()));
    }

    let method = req.method().as_str();
    let path = req.uri().path().as_str();
    let message = format!("v1.{timestamp_str}.{method}.{path}.{body_hash}");

    if !verify_device_signature(public_key, &message, signature) {
        return Err((Status::Unauthorized, "Invalid signature".to_string()));
    }

    Ok(())
}

pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Some(device_id) = req.routed_segment(1).map(|s: &str| s.to_string()) else {
            return error_outcome(req, Status::BadRequest, "Missing device_id");
        };

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return error_outcome(req, Status::InternalServerError, "Database not available");
        };

        let Ok(mut db_client) = database.client() else {
            return error_outcome(req, Status::InternalServerError, "Database error");
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return error_outcome(req, Status::NotFound, "Device not found");
        };

        if let Err((status, msg)) = verify_signature(req, &device_row) {
            return error_outcome(req, status, &msg);
        }

        Success(AuthenticatedDevice { device_row })
    }
}

pub struct AuthenticatedDeviceWallet {
    pub device_row: DeviceRow,
    pub wallet_id: i32,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDeviceWallet {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Some(device_id) = req.routed_segment(1).map(|s: &str| s.to_string()) else {
            return error_outcome(req, Status::BadRequest, "Missing device_id");
        };

        let Some(wallet_id_str) = req.routed_segment(3).map(|s: &str| s.to_string()) else {
            return error_outcome(req, Status::BadRequest, "Missing wallet_id");
        };

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return error_outcome(req, Status::InternalServerError, "Database not available");
        };

        let Ok(mut db_client) = database.client() else {
            return error_outcome(req, Status::InternalServerError, "Database error");
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return error_outcome(req, Status::NotFound, "Device not found");
        };

        if let Err((status, msg)) = verify_signature(req, &device_row) {
            return error_outcome(req, status, &msg);
        }

        let Ok(wallet_row) = WalletsStore::get_wallet(&mut db_client, &wallet_id_str) else {
            return error_outcome(req, Status::NotFound, "Wallet not found");
        };

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
        })
    }
}
