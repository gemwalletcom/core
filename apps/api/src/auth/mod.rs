mod guard;

pub use guard::{Authenticated, VerifiedAuth};

use crate::responders::{ApiError, ApiResponse};
use gem_auth::AuthClient;
use primitives::AuthNonce;
use rocket::{State, get};
use std::sync::Arc;

#[get("/devices/<device_id>/auth/nonce")]
pub async fn get_auth_nonce(device_id: &str, client: &State<Arc<AuthClient>>) -> Result<ApiResponse<AuthNonce>, ApiError> {
    Ok(client.get_nonce(device_id).await?.into())
}
