pub mod cacher;
pub mod client;
pub mod guard;
use crate::assets::AssetsClient;
use crate::notifications::NotificationsClient;
use crate::params::DeviceParam;
use crate::responders::{ApiError, ApiResponse};
use crate::scan::ScanClient;
use crate::transactions::TransactionsClient;
use crate::wallets::WalletsClient;
pub use cacher::DeviceCacher;
pub use client::DevicesClient;
use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use nft::NFTClient;
use pricer::PriceAlertClient;
use primitives::device::Device;
use primitives::rewards::{RedemptionRequest, RedemptionResult, RewardRedemptionOption};
use primitives::{
    AssetId, InAppNotification, NFTData, PriceAlerts, ReferralLeaderboard, ReportNft, RewardEvent, Rewards, ScanTransaction, ScanTransactionPayload, TransactionsResponse,
    WalletSubscription, WalletSubscriptionChains,
};
use rocket::{State, delete, get, post, put, serde::json::Json, tokio::sync::Mutex};

use crate::auth::WalletSigned;
use crate::referral::{RewardsClient, RewardsRedemptionClient};

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(device: DeviceParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.add_device(device.0)?.into())
}

#[get("/devices/<_device_id>")]
pub async fn get_device(_device_id: &str, device: AuthenticatedDevice) -> ApiResponse<Device> {
    device.device_row.as_primitive().into()
}

#[put("/devices/<_device_id>", format = "json", data = "<device_param>")]
pub async fn update_device(
    _device_id: &str,
    _device: AuthenticatedDevice,
    device_param: DeviceParam,
    client: &State<Mutex<DevicesClient>>,
) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.update_device(device_param.0)?.into())
}

#[post("/devices/<_device_id>/push-notification")]
pub async fn send_push_notification_device(_device_id: &str, device: AuthenticatedDevice, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(ApiResponse::from(
        client
            .lock()
            .await
            .send_push_notification_device(&device.device_row.device_id)
            .await
            .map_err(ApiError::from)?,
    ))
}

#[delete("/devices/<_device_id>")]
pub async fn delete_device(_device_id: &str, device: AuthenticatedDevice, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_device(&device.device_row.device_id)?.into())
}

#[get("/devices/<device_id>/is_registered")]
pub async fn is_device_registered(device_id: &str, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.is_device_registered(device_id)?.into())
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

#[get("/devices/<_device_id>/wallets/<_wallet_id>/rewards")]
pub async fn get_device_rewards(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.get_rewards_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/<_device_id>/wallets/<_wallet_id>/rewards/events")]
pub async fn get_device_rewards_events(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.lock().await.get_rewards_events_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/<_device_id>/rewards/leaderboard")]
pub async fn get_device_rewards_leaderboard(
    _device_id: &str,
    _device: AuthenticatedDevice,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<ReferralLeaderboard>, ApiError> {
    Ok(client.lock().await.get_rewards_leaderboard()?.into())
}

#[get("/devices/<_device_id>/rewards/redemptions/<code>")]
pub async fn get_device_rewards_redemption_option(
    _device_id: &str,
    code: String,
    _device: AuthenticatedDevice,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<RewardRedemptionOption>, ApiError> {
    Ok(client.lock().await.get_rewards_redemption_option(&code)?.into())
}

#[post("/devices/<_device_id>/wallets/<_wallet_id>/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_device_referral(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    request: WalletSigned<primitives::ReferralCode>,
    ip: std::net::IpAddr,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Rewards>, ApiError> {
    let wallet_identifier = primitives::WalletId::Multicoin(request.address.clone()).id();
    Ok(client
        .lock()
        .await
        .create_username(
            &wallet_identifier,
            &request.data.code,
            device.device_row.id,
            &ip.to_string(),
            device.device_row.locale.as_str(),
        )
        .await?
        .into())
}

#[post("/devices/<_device_id>/wallets/<_wallet_id>/rewards/referrals/use", format = "json", data = "<request>")]
pub async fn use_device_referral_code(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    request: WalletSigned<primitives::ReferralCode>,
    ip: std::net::IpAddr,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    let events = client
        .lock()
        .await
        .use_referral_code(&device.device_row, &request.address, &request.data.code, &ip.to_string())
        .await?;
    Ok(events.into())
}

#[post("/devices/<_device_id>/wallets/<_wallet_id>/rewards/redeem", format = "json", data = "<request>")]
pub async fn redeem_device_rewards(
    _device_id: &str,
    _wallet_id: &str,
    device: AuthenticatedDeviceWallet,
    request: WalletSigned<RedemptionRequest>,
    client: &State<Mutex<RewardsRedemptionClient>>,
) -> Result<ApiResponse<RedemptionResult>, ApiError> {
    Ok(client
        .lock()
        .await
        .redeem_by_wallet_id(device.wallet_id, &request.data.id, device.device_row.id)
        .await?
        .into())
}

#[post("/devices/<_device_id>/nft/report", format = "json", data = "<request>")]
pub async fn report_device_nft(_device_id: &str, device: AuthenticatedDevice, request: Json<ReportNft>, client: &State<Mutex<NFTClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client
        .lock()
        .await
        .report_nft(
            &device.device_row.device_id,
            request.collection_id.clone(),
            request.asset_id.clone(),
            request.reason.clone(),
        )?
        .into())
}

#[post("/devices/<_device_id>/scan/transaction", data = "<request>")]
pub async fn scan_device_transaction(
    _device_id: &str,
    _device: AuthenticatedDevice,
    request: Json<ScanTransactionPayload>,
    client: &State<Mutex<ScanClient>>,
) -> Result<ApiResponse<ScanTransaction>, ApiError> {
    Ok(client.lock().await.get_scan_transaction(request.0).await?.into())
}
