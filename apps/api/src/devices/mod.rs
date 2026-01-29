pub mod cacher;
pub mod client;
pub mod guard;
use crate::assets::AssetsClient;
use crate::notifications::NotificationsClient;
use crate::params::{DeviceIdParam, DeviceParam};
use crate::responders::{ApiError, ApiResponse};
use crate::transactions::TransactionsClient;
use crate::wallets::WalletsClient;
pub use cacher::DeviceCacher;
pub use client::DevicesClient;
use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use nft::NFTClient;
use pricer::PriceAlertClient;
use primitives::device::Device;
use primitives::{AssetId, InAppNotification, NFTData, PriceAlerts, TransactionsResponse, WalletSubscription, WalletSubscriptionChains};
use rocket::{State, delete, get, post, put, serde::json::Json, tokio::sync::Mutex};

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(device: DeviceParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.add_device(device.0)?.into())
}

#[get("/devices/<device_id>")]
pub async fn get_device(device_id: DeviceIdParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.get_device(&device_id.0)?.into())
}

#[put("/devices/<device_id>", format = "json", data = "<device>")]
pub async fn update_device(device: DeviceParam, #[allow(unused)] device_id: DeviceIdParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.update_device(device.0)?.into())
}

#[post("/devices/<device_id>/push-notification")]
pub async fn send_push_notification_device(device_id: DeviceIdParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(ApiResponse::from(
        client.lock().await.send_push_notification_device(&device_id.0).await.map_err(ApiError::from)?,
    ))
}

#[delete("/devices/<device_id>")]
pub async fn delete_device(device_id: DeviceIdParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_device(&device_id.0)?.into())
}

#[get("/devices/<_device_id>/is_registered")]
pub async fn is_device_registered(_device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.is_device_registered(_device_id)?.into())
}

#[get("/devices/<_device_id>/price_alerts?<asset_id>")]
pub async fn get_device_price_alerts(
    _device_id: &str,
    device: AuthenticatedDevice,
    asset_id: Option<&str>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.lock().await.get_price_alerts(&device.device_row.device_id, asset_id).await?.into())
}

#[post("/devices/<_device_id>/price_alerts", format = "json", data = "<subscriptions>")]
pub async fn add_device_price_alerts(
    _device_id: &str,
    device: AuthenticatedDevice,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_price_alerts(&device.device_row.device_id, subscriptions.0).await?.into())
}

#[delete("/devices/<_device_id>/price_alerts", format = "json", data = "<subscriptions>")]
pub async fn delete_device_price_alerts(
    _device_id: &str,
    device: AuthenticatedDevice,
    subscriptions: Json<PriceAlerts>,
    client: &State<Mutex<PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_price_alerts(&device.device_row.device_id, subscriptions.0).await?.into())
}

#[get("/devices/<_device_id>/subscriptions")]
pub async fn get_device_subscriptions(
    _device_id: &str,
    device: AuthenticatedDevice,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.lock().await.get_subscriptions(&device.device_row.device_id).await?.into())
}

#[post("/devices/<_device_id>/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn add_device_subscriptions(
    _device_id: &str,
    device: AuthenticatedDevice,
    subscriptions: Json<Vec<WalletSubscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_subscriptions(&device.device_row.device_id, subscriptions.0).await?.into())
}

#[delete("/devices/<_device_id>/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn delete_device_subscriptions(
    _device_id: &str,
    device: AuthenticatedDevice,
    subscriptions: Json<Vec<WalletSubscription>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_subscriptions(&device.device_row.device_id, subscriptions.0).await?.into())
}

#[get("/devices/<_device_id>/wallets/<_wallet_id>/transactions?<asset_id>&<from_timestamp>")]
pub async fn get_device_transactions(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    asset_id: Option<&str>,
    from_timestamp: Option<u64>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client
        .lock()
        .await
        .get_transactions_by_wallet_id(
            &device.device_row.device_id,
            device.device_row.id,
            device.wallet_id,
            asset_id.map(|s| s.to_string()),
            from_timestamp,
        )?
        .into())
}

#[get("/devices/<_device_id>/wallets/<_wallet_id>/assets?<from_timestamp>")]
pub async fn get_device_assets(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    from_timestamp: Option<u64>,
    client: &State<Mutex<AssetsClient>>,
) -> Result<ApiResponse<Vec<AssetId>>, ApiError> {
    Ok(client.lock().await.get_assets_by_wallet_id(device.device_row.id, device.wallet_id, from_timestamp)?.into())
}

#[get("/devices/<_device_id>/wallets/<_wallet_id>/nft_assets")]
pub async fn get_device_nft_assets(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    client: &State<Mutex<NFTClient>>,
) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.lock().await.get_nft_assets_by_wallet_id(device.device_row.id, device.wallet_id).await?.into())
}

#[get("/devices/<_device_id>/notifications?<from_timestamp>")]
pub async fn get_device_notifications(
    _device_id: &str,
    device: AuthenticatedDevice,
    from_timestamp: Option<u64>,
    client: &State<Mutex<NotificationsClient>>,
) -> Result<ApiResponse<Vec<InAppNotification>>, ApiError> {
    Ok(client.lock().await.get_notifications(&device.device_row.device_id, from_timestamp)?.into())
}

#[post("/devices/<_device_id>/notifications/read")]
pub async fn mark_device_notifications_read(_device_id: &str, device: AuthenticatedDevice, client: &State<Mutex<NotificationsClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.mark_all_as_read(&device.device_row.device_id)?.into())
}
