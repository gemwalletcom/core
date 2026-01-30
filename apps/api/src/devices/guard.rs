use rocket::Request;
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromRequest, Outcome};
use storage::Database;
use storage::database::devices::DevicesStore;
use storage::database::wallets::WalletsStore;
use storage::models::DeviceRow;

use crate::devices::constants::{HEADER_DEVICE_ID, HEADER_WALLET_ID};
use crate::devices::error::DeviceError;
use crate::responders::cache_error;

fn auth_error_outcome<T>(req: &Request<'_>, error: DeviceError) -> Outcome<T, String> {
    let status = match error {
        DeviceError::MissingHeader(_) => Status::Unauthorized,
        DeviceError::MissingParameter(_) => Status::BadRequest,
        DeviceError::DeviceNotFound | DeviceError::WalletNotFound => Status::NotFound,
        DeviceError::DatabaseUnavailable | DeviceError::DatabaseError => Status::InternalServerError,
    };
    let message = error.to_string();
    cache_error(req, &message);
    Error((status, message))
}

pub struct AuthenticatedDevice {
    pub device_row: DeviceRow,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDevice {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Some(device_id) = req.headers().get_one(HEADER_DEVICE_ID) else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_DEVICE_ID));
        };

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable);
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError);
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound);
        };

        Success(AuthenticatedDevice { device_row })
    }
}

pub struct DeviceId(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DeviceId {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        match req.headers().get_one(HEADER_DEVICE_ID) {
            Some(id) => Success(DeviceId(id.to_string())),
            None => auth_error_outcome(req, DeviceError::MissingHeader(HEADER_DEVICE_ID)),
        }
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
        let Some(device_id) = req.headers().get_one(HEADER_DEVICE_ID) else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_DEVICE_ID));
        };

        let Some(wallet_id_str) = req.headers().get_one(HEADER_WALLET_ID) else {
            return auth_error_outcome(req, DeviceError::MissingHeader(HEADER_WALLET_ID));
        };

        let Success(database) = req.guard::<&rocket::State<Database>>().await else {
            return auth_error_outcome(req, DeviceError::DatabaseUnavailable);
        };

        let Ok(mut db_client) = database.client() else {
            return auth_error_outcome(req, DeviceError::DatabaseError);
        };

        let Ok(device_row) = DevicesStore::get_device(&mut db_client, &device_id) else {
            return auth_error_outcome(req, DeviceError::DeviceNotFound);
        };

        let Ok(wallet_row) = WalletsStore::get_wallet(&mut db_client, &wallet_id_str) else {
            return auth_error_outcome(req, DeviceError::WalletNotFound);
        };

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
        })
    }
}
