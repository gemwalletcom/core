use primitives::WalletId;
use rocket::Request;
use rocket::outcome::Outcome::Success;
use rocket::request::{FromRequest, Outcome};
use storage::models::DeviceRow;

use super::auth::{auth_error_outcome, authenticate, lookup_device_wallet};
use crate::devices::error::DeviceError;

// Verifies control of the device key, then resolves a wallet attached to that device.
// This proves device-wallet scope, not wallet-owner intent; routes that need owner approval must also use WalletSigned<T>.
pub struct AuthenticatedDeviceWallet {
    pub device_row: DeviceRow,
    pub wallet_id: i32,
    pub wallet_identifier: WalletId,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedDeviceWallet {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let auth = match authenticate(req).await {
            Ok(auth) => auth,
            Err(error) => return error,
        };

        let Some(wallet_id_str) = auth.wallet_id else {
            return auth_error_outcome(req, DeviceError::MissingWalletId, Some(&auth.device_id), None);
        };

        let (device_row, wallet_row) = match lookup_device_wallet(req, &auth.device_id, &wallet_id_str).await {
            Ok(result) => result,
            Err(error) => return error,
        };

        Success(AuthenticatedDeviceWallet {
            device_row,
            wallet_id: wallet_row.id,
            wallet_identifier: wallet_row.wallet_id.0,
        })
    }
}
