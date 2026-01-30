use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use crate::responders::{cache_error, verify_request_signature};

fn error_outcome<T>(req: &Request<'_>, status: Status, message: &str) -> Outcome<T, String> {
    cache_error(req, message);
    Error((status, message.to_string()))
}

fn verify_signature(req: &Request<'_>, device_row: &DeviceRow) -> Result<(), (Status, String)> {
    let Some(ref public_key) = device_row.public_key else {
        return Ok(());
    };
    verify_request_signature(req, public_key)
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
