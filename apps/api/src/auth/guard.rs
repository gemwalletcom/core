use crate::responders::ErrorContext;
use gem_auth::{AuthClient, verify_auth_signature};
use primitives::{AuthMessage, AuthenticatedRequest, WalletId};
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::{Data, Request, State};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use storage::database::devices::DevicesStore;
use storage::database::devices_sessions::DeviceSessionsStore;
use storage::models::{DeviceRow, NewDeviceSessionRow, NewWalletRow};
use storage::repositories::wallets_repository::WalletsRepository;
use storage::{Database, WalletSource, WalletType};

fn error_outcome<'r, T>(req: &'r Request<'_>, status: Status, message: &str) -> Outcome<'r, T, String> {
    req.local_cache(|| ErrorContext(message.to_string()));
    Error((status, message.to_string()))
}

pub struct VerifiedAuth {
    pub device: DeviceRow,
    pub address: String,
}

pub struct Authenticated<T> {
    pub auth: VerifiedAuth,
    pub data: T,
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned + Send> FromData<'r> for Authenticated<T> {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let Success(auth_client) = req.guard::<&State<Arc<AuthClient>>>().await else {
            return error_outcome(req, Status::InternalServerError, "Auth client not available");
        };
        let Success(database) = req.guard::<&State<Database>>().await else {
            return error_outcome(req, Status::InternalServerError, "Database not available");
        };

        let Ok(bytes) = data.open(1.mebibytes()).into_bytes().await else {
            return error_outcome(req, Status::BadRequest, "Failed to read body");
        };
        if !bytes.is_complete() {
            return error_outcome(req, Status::BadRequest, "Request body too large");
        }

        let Ok(body) = serde_json::from_slice::<AuthenticatedRequest<T>>(&bytes.into_inner()) else {
            return error_outcome(req, Status::BadRequest, "Invalid JSON");
        };

        let Ok(auth_nonce) = auth_client.get_auth_nonce(&body.auth.device_id, &body.auth.nonce).await else {
            return error_outcome(req, Status::Unauthorized, "Invalid nonce");
        };

        let Ok(mut db_client) = database.client() else {
            return error_outcome(req, Status::InternalServerError, "Database error");
        };
        let Ok(device) = DevicesStore::get_device(&mut db_client, &body.auth.device_id) else {
            return error_outcome(req, Status::Unauthorized, "Device not found");
        };

        let auth_message = AuthMessage {
            chain: body.auth.chain,
            address: body.auth.address.clone(),
            auth_nonce,
        };
        if !verify_auth_signature(&auth_message, &body.auth.signature) {
            return error_outcome(req, Status::Unauthorized, "Invalid signature");
        }

        if auth_client.invalidate_nonce(&body.auth.device_id, &body.auth.nonce).await.is_err() {
            return error_outcome(req, Status::InternalServerError, "Failed to invalidate nonce");
        }

        let wallet_identifier = WalletId::Multicoin(body.auth.address.clone()).id();
        let wallet = match db_client.get_or_create_wallet(NewWalletRow {
            identifier: wallet_identifier,
            wallet_type: WalletType(primitives::WalletType::Multicoin),
            source: WalletSource(primitives::WalletSource::Import),
        }) {
            Ok(w) => w,
            Err(_) => return error_outcome(req, Status::InternalServerError, "Failed to get or create wallet"),
        };

        let session = NewDeviceSessionRow {
            device_id: device.id,
            wallet_id: wallet.id,
            signature: body.auth.signature.clone(),
        };
        if DeviceSessionsStore::add_device_session(&mut db_client, session).is_err() {
            return error_outcome(req, Status::InternalServerError, "Failed to store session");
        }

        Success(Authenticated {
            auth: VerifiedAuth {
                device,
                address: body.auth.address,
            },
            data: body.data,
        })
    }
}
