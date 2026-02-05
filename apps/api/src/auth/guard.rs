use crate::responders::cache_error;
use gem_auth::{AuthClient, verify_auth_signature};
use primitives::{AuthMessage, AuthenticatedRequest};
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::{Data, Request, State};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use storage::Database;
use storage::database::devices::DevicesStore;
use storage::models::DeviceRow;

fn error_outcome<'r, T>(req: &'r Request<'_>, status: Status, message: &str) -> Outcome<'r, T, String> {
    cache_error(req, message);
    Error((status, message.to_string()))
}

struct VerifiedBody<T> {
    device_id: String,
    address: String,
    data: T,
}

async fn verify_wallet_signature<'r, T: DeserializeOwned + Send, O>(req: &'r Request<'_>, data: Data<'r>) -> Result<VerifiedBody<T>, Outcome<'r, O, String>> {
    let Success(auth_client) = req.guard::<&State<Arc<AuthClient>>>().await else {
        return Err(error_outcome(req, Status::InternalServerError, "Auth client not available"));
    };

    let Ok(bytes) = data.open(8.mebibytes()).into_bytes().await else {
        return Err(error_outcome(req, Status::BadRequest, "Failed to read body"));
    };
    if !bytes.is_complete() {
        return Err(error_outcome(req, Status::BadRequest, "Request body too large"));
    }

    let raw_body = bytes.into_inner();

    if let Some(expected_hash) = req.headers().get_one("x-device-body-hash") {
        let actual_hash = format!("{:x}", Sha256::digest(&raw_body));
        if actual_hash != expected_hash {
            return Err(error_outcome(req, Status::BadRequest, "Body hash mismatch"));
        }
    }

    let Ok(body) = serde_json::from_slice::<AuthenticatedRequest<T>>(&raw_body) else {
        return Err(error_outcome(req, Status::BadRequest, "Invalid JSON"));
    };

    let Ok(auth_nonce) = auth_client.get_auth_nonce(&body.auth.device_id, &body.auth.nonce).await else {
        return Err(error_outcome(req, Status::Unauthorized, "Invalid nonce"));
    };

    let auth_message = AuthMessage {
        chain: body.auth.chain,
        address: body.auth.address.clone(),
        auth_nonce,
    };
    if !verify_auth_signature(&auth_message, &body.auth.signature) {
        return Err(error_outcome(req, Status::Unauthorized, "Invalid signature"));
    }

    if auth_client.invalidate_nonce(&body.auth.device_id, &body.auth.nonce).await.is_err() {
        return Err(error_outcome(req, Status::InternalServerError, "Failed to invalidate nonce"));
    }

    Ok(VerifiedBody {
        device_id: body.auth.device_id,
        address: body.auth.address,
        data: body.data,
    })
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
        let verified = match verify_wallet_signature(req, data).await {
            Ok(v) => v,
            Err(outcome) => return outcome,
        };

        let Success(database) = req.guard::<&State<Database>>().await else {
            return error_outcome(req, Status::InternalServerError, "Database not available");
        };
        let Ok(mut db_client) = database.client() else {
            return error_outcome(req, Status::InternalServerError, "Database error");
        };
        let Ok(device) = DevicesStore::get_device(&mut db_client, &verified.device_id) else {
            return error_outcome(req, Status::Unauthorized, "Device not found");
        };

        Success(Authenticated {
            auth: VerifiedAuth {
                device,
                address: verified.address,
            },
            data: verified.data,
        })
    }
}

pub struct WalletSigned<T> {
    pub address: String,
    pub data: T,
}

#[rocket::async_trait]
impl<'r, T: DeserializeOwned + Send> FromData<'r> for WalletSigned<T> {
    type Error = String;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        let verified = match verify_wallet_signature(req, data).await {
            Ok(v) => v,
            Err(outcome) => return outcome,
        };

        if let Some(url_device_id) = req.routed_segment(1).map(|s: &str| s.to_string())
            && url_device_id != verified.device_id
        {
            return error_outcome(req, Status::Unauthorized, "Device ID mismatch");
        }

        Success(WalletSigned {
            address: verified.address,
            data: verified.data,
        })
    }
}
