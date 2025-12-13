use gem_auth::{AuthClient, verify_auth_signature};
use primitives::{AuthMessage, AuthenticatedRequest};
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::{Data, Request, State};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use storage::Database;

pub struct VerifiedAuth {
    pub device_id: String,
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
            return Error((Status::InternalServerError, "Auth client not available".into()));
        };
        let Success(database) = req.guard::<&State<Database>>().await else {
            return Error((Status::InternalServerError, "Database not available".into()));
        };

        let Ok(bytes) = data.open(1.mebibytes()).into_bytes().await else {
            return Error((Status::BadRequest, "Failed to read body".into()));
        };
        if !bytes.is_complete() {
            return Error((Status::BadRequest, "Request body too large".into()));
        }

        let Ok(body) = serde_json::from_slice::<AuthenticatedRequest<T>>(&bytes.into_inner()) else {
            return Error((Status::BadRequest, "Invalid JSON".into()));
        };

        let Ok(auth_nonce) = auth_client.get_auth_nonce(&body.auth.device_id, &body.auth.nonce).await else {
            return Error((Status::Unauthorized, "Invalid nonce".into()));
        };

        let Ok(mut db_client) = database.client() else {
            return Error((Status::InternalServerError, "Database error".into()));
        };
        if db_client.get_device(&body.auth.device_id).is_err() {
            return Error((Status::Unauthorized, "Device not found".into()));
        }

        let auth_message = AuthMessage {
            chain: body.auth.chain,
            address: body.auth.address.clone(),
            auth_nonce,
        };
        if !verify_auth_signature(&auth_message, &body.auth.signature) {
            return Error((Status::Unauthorized, "Invalid signature".into()));
        }

        if auth_client.invalidate_nonce(&body.auth.device_id, &body.auth.nonce).await.is_err() {
            return Error((Status::InternalServerError, "Failed to invalidate nonce".into()));
        }

        Success(Authenticated {
            auth: VerifiedAuth {
                device_id: body.auth.device_id,
                address: body.auth.address,
            },
            data: body.data,
        })
    }
}
