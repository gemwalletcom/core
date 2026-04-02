pub mod auth_config;
pub mod client;
pub mod clients;
pub mod constants;
pub mod error;
pub mod guard;
pub mod signature;
use crate::assets::AssetsClient;
use crate::metrics::fiat::FiatMetrics;
use crate::params::{
    AssetIdParam, ChainParam, ChartPeriodParam, CurrencyParam, DeviceIdParam, DeviceParam, FiatProviderIdParam, FiatQuoteTypeParam, TransactionIdParam, UserAgent,
};
use crate::responders::{ApiError, ApiResponse};
use auth_config::AuthConfig;
pub use client::DevicesClient;
pub(crate) use clients::WalletSubscriptionInput;
pub use clients::{
    AddressNamesClient, FiatQuotesClient, NotificationsClient, PortfolioClient, RewardsClient, RewardsRedemptionClient, ScanClient, ScanProviderFactory, TransactionsClient,
    WalletsClient,
};
use gem_auth::AuthClient;
use guard::{AuthenticatedDevice, AuthenticatedDeviceWallet, VerifiedDeviceId};
use name_resolver::client::Client as NameClient;
use nft::NFTClient;
use primitives::DeviceToken;
use primitives::device::Device;
use primitives::name::NameRecord;
use primitives::rewards::{RedemptionRequest, RedemptionResult, RewardRedemptionOption};
use primitives::{
    AddressName, AssetId, AuthNonce, ChainAddress, FiatAssets, FiatQuote, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuotes, InAppNotification, MigrateDeviceIdRequest,
    NFTData, PortfolioAssets, PortfolioAssetsRequest, PriceAlerts, ReportNft, RewardEvent, Rewards, ScanTransaction, ScanTransactionPayload, Transaction, TransactionsResponse,
    WalletSubscriptionChains,
};
use rocket::{State, delete, get, post, put, serde::json::Json, tokio::sync::Mutex};
use std::sync::Arc;

use crate::auth::WalletSigned;

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device(device: DeviceParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.add_device(device.0)?.into())
}

#[post("/devices", format = "json", data = "<device>")]
pub async fn add_device_v2(_device_id: VerifiedDeviceId, device: DeviceParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
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

#[delete("/devices/<device_id>")]
pub async fn delete_device(device_id: DeviceIdParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_device(&device_id.0)?.into())
}

#[get("/devices")]
pub async fn get_device_v2(device: AuthenticatedDevice, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.get_device(&device.device_row.device_id)?.into())
}

#[delete("/devices")]
pub async fn delete_device_v2(device: AuthenticatedDevice, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_device(&device.device_row.device_id)?.into())
}

#[get("/devices/is_registered")]
pub async fn is_device_registered_v2(device_id: VerifiedDeviceId, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.is_device_registered(&device_id.0)?.into())
}

#[post("/devices/migrate", format = "json", data = "<request>")]
pub async fn migrate_device_id_v2(request: Json<MigrateDeviceIdRequest>, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.migrate_device_id(&request.old_device_id, &request.public_key)?.into())
}

#[get("/devices/assets?<from_timestamp>")]
pub async fn get_device_assets_v2(
    device: AuthenticatedDeviceWallet,
    from_timestamp: Option<u64>,
    client: &State<Mutex<AssetsClient>>,
) -> Result<ApiResponse<Vec<AssetId>>, ApiError> {
    Ok(client.lock().await.get_assets_by_wallet_id(device.device_row.id, device.wallet_id, from_timestamp)?.into())
}

#[get("/devices/transactions?<asset_id>&<from_timestamp>")]
pub async fn get_device_transactions_v2(
    device: AuthenticatedDeviceWallet,
    asset_id: Option<AssetIdParam>,
    from_timestamp: Option<u64>,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client
        .lock()
        .await
        .get_transactions_by_wallet_id(&device.device_row.device_id, device.device_row.id, device.wallet_id, asset_id.map(|x| x.0), from_timestamp)
        .await?
        .into())
}

#[get("/devices/transactions/<id>")]
pub async fn get_device_transaction_by_id_v2(
    _device: AuthenticatedDevice,
    id: TransactionIdParam,
    client: &State<Mutex<TransactionsClient>>,
) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.lock().await.get_transaction_by_id(&id.0)?.into())
}

#[post("/devices/address_names", format = "json", data = "<requests>")]
pub async fn get_device_address_names_v2(
    _device: AuthenticatedDevice,
    requests: Json<Vec<ChainAddress>>,
    client: &State<Mutex<AddressNamesClient>>,
) -> Result<ApiResponse<Vec<AddressName>>, ApiError> {
    Ok(client.lock().await.get_address_names(requests.into_inner())?.into())
}

#[get("/devices/nft_assets")]
pub async fn get_device_nft_assets_v2(device: AuthenticatedDeviceWallet, client: &State<Mutex<NFTClient>>) -> Result<ApiResponse<Vec<NFTData>>, ApiError> {
    Ok(client.lock().await.get_nft_assets_by_wallet_id(device.device_row.id, device.wallet_id).await?.into())
}

#[get("/devices/rewards")]
pub async fn get_device_rewards_v2(device: AuthenticatedDeviceWallet, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Rewards>, ApiError> {
    Ok(client.lock().await.get_rewards_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/rewards/events")]
pub async fn get_device_rewards_events_v2(device: AuthenticatedDeviceWallet, client: &State<Mutex<RewardsClient>>) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    Ok(client.lock().await.get_rewards_events_by_wallet_id(device.wallet_id)?.into())
}

#[get("/devices/rewards/redemptions/<code>")]
pub async fn get_device_rewards_redemption_v2(
    _device: AuthenticatedDevice,
    code: &str,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<RewardRedemptionOption>, ApiError> {
    Ok(client.lock().await.get_rewards_redemption_option(code)?.into())
}

#[post("/devices/rewards/referrals/create", format = "json", data = "<request>")]
pub async fn create_device_referral_v2(
    device: AuthenticatedDevice,
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

#[post("/devices/rewards/referrals/use", format = "json", data = "<request>")]
pub async fn use_device_referral_code_v2(
    device: AuthenticatedDevice,
    request: WalletSigned<primitives::ReferralCode>,
    ip: std::net::IpAddr,
    user_agent: UserAgent,
    client: &State<Mutex<RewardsClient>>,
) -> Result<ApiResponse<Vec<RewardEvent>>, ApiError> {
    let events = client
        .lock()
        .await
        .use_referral_code(&device.device_row, &request.address, &request.data.code, &ip.to_string(), &user_agent.0)
        .await?;
    Ok(events.into())
}

#[post("/devices/rewards/redeem", format = "json", data = "<request>")]
pub async fn redeem_device_rewards_v2(
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

#[put("/devices", format = "json", data = "<device_param>")]
pub async fn update_device_v2(_device: AuthenticatedDevice, device_param: DeviceParam, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.lock().await.update_device(device_param.0)?.into())
}

#[post("/devices/push-notification")]
pub async fn send_push_notification_device_v2(device: AuthenticatedDevice, client: &State<Mutex<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(ApiResponse::from(
        client
            .lock()
            .await
            .send_push_notification_device(&device.device_row.device_id)
            .await
            .map_err(ApiError::from)?,
    ))
}

#[post("/devices/nft/report", format = "json", data = "<request>")]
pub async fn report_device_nft_v2(device: AuthenticatedDevice, request: Json<ReportNft>, client: &State<Mutex<NFTClient>>) -> Result<ApiResponse<bool>, ApiError> {
    let asset_id = request
        .asset_id
        .as_deref()
        .map(|asset_id| AssetId::new(asset_id).ok_or_else(|| ApiError::BadRequest(format!("Invalid asset_id: {asset_id}"))))
        .transpose()?;

    Ok(client
        .lock()
        .await
        .report_nft(&device.device_row.device_id, request.collection_id.clone(), asset_id, request.reason.clone())?
        .into())
}

#[get("/devices/name/resolve/<name>?<chain>")]
pub async fn get_device_name_resolve_v2(
    _device: AuthenticatedDevice,
    name: &str,
    chain: ChainParam,
    client: &State<Mutex<NameClient>>,
) -> Result<ApiResponse<Option<NameRecord>>, ApiError> {
    let result = client.lock().await.resolve(name, chain.0).await;
    match result {
        Ok(record) => Ok(Some(record).into()),
        Err(_) => Ok(None.into()),
    }
}

#[post("/devices/scan/transaction", data = "<request>")]
pub async fn scan_device_transaction_v2(
    _device: AuthenticatedDevice,
    request: Json<ScanTransactionPayload>,
    client: &State<Mutex<ScanClient>>,
) -> Result<ApiResponse<ScanTransaction>, ApiError> {
    Ok(client.lock().await.get_scan_transaction(request.0).await?.into())
}

#[get("/devices/notifications?<from_timestamp>")]
pub async fn get_device_notifications_v2(
    device: AuthenticatedDevice,
    from_timestamp: Option<u64>,
    client: &State<Mutex<NotificationsClient>>,
) -> Result<ApiResponse<Vec<InAppNotification>>, ApiError> {
    Ok(client.lock().await.get_notifications(&device.device_row.device_id, from_timestamp)?.into())
}

#[post("/devices/notifications/read")]
pub async fn mark_device_notifications_read_v2(device: AuthenticatedDevice, client: &State<Mutex<NotificationsClient>>) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.mark_all_as_read(&device.device_row.device_id)?.into())
}

#[get("/devices/subscriptions")]
pub async fn get_device_subscriptions_v2(device: AuthenticatedDevice, client: &State<Mutex<WalletsClient>>) -> Result<ApiResponse<Vec<WalletSubscriptionChains>>, ApiError> {
    Ok(client.lock().await.get_subscriptions(device.device_row.id)?.into())
}

#[post("/devices/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn add_device_subscriptions_v2(
    device: AuthenticatedDevice,
    subscriptions: Json<Vec<WalletSubscriptionInput>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    let wallet_subscriptions = subscriptions.0.into_iter().map(|x| x.into_wallet_subscription()).collect();
    Ok(client.lock().await.add_subscriptions(device.device_row.id, wallet_subscriptions).await?.into())
}

#[delete("/devices/subscriptions", format = "json", data = "<subscriptions>")]
pub async fn delete_device_subscriptions_v2(
    device: AuthenticatedDevice,
    subscriptions: Json<Vec<WalletSubscriptionChains>>,
    client: &State<Mutex<WalletsClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_subscriptions(device.device_row.id, subscriptions.0).await?.into())
}

#[get("/devices/auth/nonce")]
pub async fn get_auth_nonce_v2(device: AuthenticatedDevice, client: &State<Arc<AuthClient>>) -> Result<ApiResponse<AuthNonce>, ApiError> {
    Ok(client.get_nonce(&device.device_row.device_id).await?.into())
}

#[get("/devices/token")]
pub async fn get_device_token_v2(device: AuthenticatedDevice, config: &State<AuthConfig>, client: &State<Arc<AuthClient>>) -> Result<ApiResponse<DeviceToken>, ApiError> {
    Ok(client.create_device_token(&device.device_row.device_id, &config.jwt.secret, config.jwt.expiry)?.into())
}

#[get("/devices/price_alerts?<asset_id>")]
pub async fn get_device_price_alerts_v2(
    device: AuthenticatedDevice,
    asset_id: Option<&str>,
    client: &State<Mutex<pricer::PriceAlertClient>>,
) -> Result<ApiResponse<PriceAlerts>, ApiError> {
    Ok(client.lock().await.get_price_alerts(&device.device_row.device_id, asset_id).await?.into())
}

#[post("/devices/price_alerts", format = "json", data = "<price_alerts>")]
pub async fn add_device_price_alerts_v2(
    device: AuthenticatedDevice,
    price_alerts: Json<PriceAlerts>,
    client: &State<Mutex<pricer::PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.add_price_alerts(&device.device_row.device_id, price_alerts.0).await?.into())
}

#[delete("/devices/price_alerts", format = "json", data = "<price_alerts>")]
pub async fn delete_device_price_alerts_v2(
    device: AuthenticatedDevice,
    price_alerts: Json<PriceAlerts>,
    client: &State<Mutex<pricer::PriceAlertClient>>,
) -> Result<ApiResponse<usize>, ApiError> {
    Ok(client.lock().await.delete_price_alerts(&device.device_row.device_id, price_alerts.0).await?.into())
}

#[get("/devices/fiat/transactions")]
pub async fn get_device_fiat_transactions_v2(
    device: AuthenticatedDeviceWallet,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<Vec<primitives::FiatTransactionData>>, ApiError> {
    Ok(client.lock().await.get_transactions_by_wallet_id(device.device_row.id, device.wallet_id)?.into())
}

#[get("/devices/fiat/assets/<quote_type>")]
pub async fn get_device_fiat_assets_v2(
    _device: AuthenticatedDevice,
    quote_type: FiatQuoteTypeParam,
    client: &State<Mutex<FiatQuotesClient>>,
) -> Result<ApiResponse<FiatAssets>, ApiError> {
    let assets = match quote_type.0 {
        FiatQuoteType::Buy => client.lock().await.get_on_ramp_assets().await?,
        FiatQuoteType::Sell => client.lock().await.get_off_ramp_assets().await?,
    };
    Ok(assets.into())
}

#[get("/devices/fiat/quotes/<quote_type>/<asset_id>?<amount>&<currency>&<provider>&<ip_address>", rank = 2)]
pub async fn get_fiat_quotes_v2(
    _device: AuthenticatedDeviceWallet,
    quote_type: FiatQuoteTypeParam,
    asset_id: AssetIdParam,
    amount: f64,
    currency: CurrencyParam,
    provider: Option<FiatProviderIdParam>,
    ip_address: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
    fiat_metrics: &State<Arc<FiatMetrics>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let fallback_ip_address = if cfg!(debug_assertions) { constants::DEBUG_FIAT_IP.to_string() } else { ip.to_string() };
    let quote_request = FiatQuoteRequest {
        asset_id: asset_id.0,
        quote_type: quote_type.0,
        amount,
        currency: currency.as_string(),
        provider_id: provider.map(|p| p.0.id().to_string()),
        ip_address: ip_address.map(str::to_string).unwrap_or(fallback_ip_address),
    };
    let quotes = client.lock().await.get_quotes(quote_request).await?;
    fiat_metrics.record_quotes(&quotes);
    Ok(quotes.into())
}

#[get("/devices/fiat/quotes/<quote_id>", rank = 2)]
pub async fn get_fiat_quote_v2(_device: AuthenticatedDevice, quote_id: &str, client: &State<Mutex<FiatQuotesClient>>) -> Result<ApiResponse<FiatQuote>, ApiError> {
    Ok(client.lock().await.get_quote(quote_id).await?.into())
}

#[get("/fiat/quotes/<quote_type>?<asset_id>&<amount>&<currency>&<provider_id>&<ip_address>")]
pub async fn get_fiat_quotes_v1(
    quote_type: FiatQuoteTypeParam,
    asset_id: AssetIdParam,
    amount: f64,
    currency: CurrencyParam,
    provider_id: Option<FiatProviderIdParam>,
    ip_address: Option<&str>,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
    fiat_metrics: &State<Arc<FiatMetrics>>,
) -> Result<ApiResponse<FiatQuotes>, ApiError> {
    let fallback_ip_address = if cfg!(debug_assertions) { constants::DEBUG_FIAT_IP.to_string() } else { ip.to_string() };
    let quote_request = FiatQuoteRequest {
        asset_id: asset_id.0,
        quote_type: quote_type.0,
        amount,
        currency: currency.as_string(),
        provider_id: provider_id.map(|p| p.0.id().to_string()),
        ip_address: ip_address.map(str::to_string).unwrap_or(fallback_ip_address),
    };
    let quotes = client.lock().await.get_quotes(quote_request).await?;
    fiat_metrics.record_quotes(&quotes);
    Ok(quotes.into())
}

#[get("/devices/fiat/quotes/<quote_id>/url", rank = 1)]
pub async fn get_fiat_quote_url_v2(
    device: AuthenticatedDeviceWallet,
    quote_id: &str,
    ip: std::net::IpAddr,
    client: &State<Mutex<FiatQuotesClient>>,
    fiat_metrics: &State<Arc<FiatMetrics>>,
) -> Result<ApiResponse<FiatQuoteUrl>, ApiError> {
    let locale = device.device_row.locale.as_str();
    let ip_address = if cfg!(debug_assertions) { constants::DEBUG_FIAT_IP.to_string() } else { ip.to_string() };
    let (url, quote) = client
        .lock()
        .await
        .get_quote_url(quote_id, device.wallet_id, device.device_row.id, &ip_address, locale)
        .await?;
    fiat_metrics.record_quote_url(&quote);
    Ok(url.into())
}

#[post("/devices/portfolio/assets?<period>", format = "json", data = "<request>")]
pub async fn get_device_portfolio_assets_v2(
    _device: AuthenticatedDevice,
    period: ChartPeriodParam,
    request: Json<PortfolioAssetsRequest>,
    portfolio_client: &State<Mutex<PortfolioClient>>,
) -> Result<ApiResponse<PortfolioAssets>, ApiError> {
    Ok(portfolio_client.lock().await.get_portfolio_charts(request.0.assets, period.0)?.into())
}
