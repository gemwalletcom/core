mod assets;
mod auth;
mod catchers;
mod chain;
mod config;
mod devices;
mod fiat;
mod markets;
mod metrics;
mod model;
mod name;
mod nft;
mod notifications;
mod params;
mod price_alerts;
mod prices;
mod referral;
mod responders;
mod scan;
mod status;
mod swap;
mod transactions;
mod wallets;
mod webhooks;
mod websocket_prices;
mod websocket_stream;

use std::{str::FromStr, sync::Arc};

use ::fiat::FiatClient;
use ::nft::{NFTClient, NFTProviderConfig};
use api_connector::PusherClient;
use assets::{AssetsClient, SearchClient};
use cacher::CacherClient;
use config::ConfigClient;
use devices::{DeviceCacher, DevicesClient};
use fiat::FiatProviderFactory;
use gem_auth::AuthClient;
use gem_rewards::{AbuseIPDBClient, IpApiClient, IpCheckProvider, IpSecurityClient};
use gem_tracing::{SentryConfig, SentryTracing};
use metrics::MetricsClient;
use model::APIService;
use name_resolver::NameProviderFactory;
use name_resolver::client::Client as NameClient;
use notifications::NotificationsClient;
use pricer::{ChartClient, MarketsClient, PriceAlertClient, PriceClient};
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket, catchers, routes};
use scan::{ScanClient, ScanProviderFactory};
use search_index::SearchIndexClient;
use settings::Settings;
use settings_chain::{ChainProviders, ProviderFactory};
use storage::Database;
use streamer::{StreamProducer, StreamProducerConfig};
use swap::SwapClient;
use transactions::TransactionsClient;
use wallets::WalletsClient;
use webhooks::WebhooksClient;
use websocket_prices::PriceObserverConfig;

fn mount_routes(rocket: Rocket<Build>, metrics_path: &str) -> Rocket<Build> {
    rocket
        .mount("/", routes![status::get_status, status::get_health])
        .mount(
            "/v1",
            routes![
                prices::get_price,
                prices::get_assets_prices,
                prices::get_charts,
                prices::get_fiat_rates,
                fiat::get_fiat_quotes_by_type,
                fiat::get_fiat_quotes,
                fiat::get_fiat_on_ramp_quotes,
                fiat::get_fiat_assets,
                fiat::get_fiat_on_ramp_assets,
                fiat::get_fiat_off_ramp_assets,
                fiat::create_fiat_webhook,
                fiat::get_fiat_order,
                fiat::get_fiat_quote_url,
                config::get_config,
                name::get_name_resolve,
                devices::add_device,
                devices::get_device,
                devices::update_device,
                devices::delete_device,
                auth::get_auth_nonce,
                assets::get_asset,
                assets::get_assets,
                assets::add_asset,
                assets::get_assets_search,
                assets::get_search,
                transactions::get_transaction_by_id,
                chain::block::get_latest_block_number,
                chain::block::get_block_transactions,
                chain::block::get_block_transactions_finalize,
                swap::get_swap_assets,
                nft::get_nft_assets_by_chain,
                nft::get_nft_collection,
                nft::get_nft_asset,
                nft::get_nft_asset_image_preview,
                nft::update_nft_collection,
                nft::update_nft_asset,
                nft::report_nft,
                price_alerts::get_price_alerts,
                price_alerts::add_price_alerts,
                price_alerts::delete_price_alerts,
                scan::scan_transaction,
                scan::get_scan_address,
                markets::get_markets,
                chain::staking::get_validators,
                chain::staking::get_staking_apy,
                chain::token::get_token,
                chain::address::get_balances,
                chain::address::get_assets,
                chain::address::get_transactions,
                webhooks::create_support_webhook,
                fiat::get_ip_address,
                referral::get_rewards_leaderboard,
                referral::get_rewards_redemption_option,
                referral::get_rewards_events,
                referral::get_rewards,
                referral::create_referral,
                referral::use_referral_code,
                referral::redeem_rewards,
                notifications::get_notifications,
                notifications::mark_notifications_read,
            ],
        )
        .mount(
            "/v2",
            routes![
                devices::get_fiat_quotes_v2,
                devices::get_fiat_quote_url_v2,
                scan::scan_transaction_v2,
                devices::add_device_v2,
                devices::get_device_v2,
                devices::delete_device_v2,
                devices::is_device_registered_v2,
                devices::migrate_device_id_v2,
                devices::update_device_v2,
                devices::send_push_notification_device_v2,
                devices::report_device_nft_v2,
                devices::scan_device_transaction_v2,
                devices::get_device_assets_v2,
                devices::get_device_transactions_v2,
                devices::get_device_nft_assets_v2,
                devices::get_device_rewards_v2,
                devices::get_device_rewards_events_v2,
                devices::create_device_referral_v2,
                devices::use_device_referral_code_v2,
                devices::redeem_device_rewards_v2,
                devices::get_device_notifications_v2,
                devices::mark_device_notifications_read_v2,
                devices::get_device_subscriptions_v2,
                devices::add_device_subscriptions_v2,
                devices::delete_device_subscriptions_v2,
                devices::get_device_price_alerts_v2,
                devices::add_device_price_alerts_v2,
                devices::delete_device_price_alerts_v2,
                devices::get_auth_nonce_v2,
                devices::get_device_token_v2,
                wallets::get_subscriptions,
                wallets::add_subscriptions,
                wallets::delete_subscriptions,
            ],
        )
        .mount(metrics_path, routes![metrics::get_metrics])
        .register("/", catchers![catchers::default_catcher])
}

async fn rocket_api(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let settings_clone = settings.clone();

    let database = Database::new(postgres_url, settings.postgres.pool);
    let cacher_client = CacherClient::new(redis_url).await;

    let price_client = PriceClient::new(database.clone(), cacher_client.clone());
    let charts_client = ChartClient::new(database.clone());
    let config_client = ConfigClient::new(database.clone());
    let price_alert_client = PriceAlertClient::new(database.clone());
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers);

    let chain_client = chain::ChainClient::new(ChainProviders::new(ProviderFactory::new_providers(&settings)));

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let pusher_client = PusherClient::new(settings.pusher.url, settings.pusher.ios.topic);
    let devices_client = DevicesClient::new(database.clone(), pusher_client.clone());
    let transactions_client = TransactionsClient::new(database.clone());
    let stream_producer = StreamProducer::new(&rabbitmq_config, "api").await.unwrap();
    let device_cacher = DeviceCacher::new(database.clone(), cacher_client.clone());
    let wallets_client = WalletsClient::new(database.clone(), device_cacher, stream_producer.clone());
    let metrics_client = MetricsClient::new();

    let security_providers = ScanProviderFactory::create_providers(&settings_clone);
    let scan_client = ScanClient::new(database.clone(), security_providers);
    let assets_client = AssetsClient::new(database.clone());
    let search_index_client = SearchIndexClient::new(&settings_clone.meilisearch.url.clone(), &settings_clone.meilisearch.key.clone());
    let search_client = SearchClient::new(&search_index_client, price_client.clone());
    let swap_client = SwapClient::new(database.clone());
    let fiat_providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let fiat_ip_check_client = FiatProviderFactory::new_ip_check_client(settings_clone.clone());
    let fiat_client = FiatClient::new(
        database.clone(),
        cacher_client.clone(),
        fiat_providers,
        fiat_ip_check_client.clone(),
        stream_producer.clone(),
    );
    let fiat_quotes_client = fiat::FiatQuotesClient::new(fiat_client);
    let nft_config = NFTProviderConfig::new(settings.nft.opensea.key.secret.clone(), settings.nft.magiceden.key.secret.clone());
    let nft_client = NFTClient::new(database.clone(), nft_config);
    let auth_client = Arc::new(AuthClient::new(cacher_client.clone()));
    let markets_client = MarketsClient::new(database.clone(), cacher_client.clone());
    let webhooks_client = WebhooksClient::new(stream_producer.clone());
    let ip_check_providers: Vec<Arc<dyn IpCheckProvider>> = vec![
        Arc::new(AbuseIPDBClient::new(settings.ip.abuseipdb.url.clone(), settings.ip.abuseipdb.key.secret.clone())),
        Arc::new(IpApiClient::new(settings.ip.ipapi.url.clone(), settings.ip.ipapi.key.secret.clone())),
    ];
    let ip_security_client = IpSecurityClient::new(ip_check_providers, cacher_client.clone());
    let rewards_client = referral::RewardsClient::new(database.clone(), stream_producer.clone(), ip_security_client, pusher_client.clone());
    let redemption_client = referral::RewardsRedemptionClient::new(database.clone(), stream_producer.clone());
    let notifications_client = NotificationsClient::new(database.clone());
    let jwt_config = devices::auth_config::JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    let auth_config = devices::auth_config::AuthConfig::new(settings.api.auth.enabled, settings.api.auth.tolerance, jwt_config);

    let rocket = rocket::build()
        .manage(auth_config)
        .manage(database)
        .manage(Mutex::new(fiat_quotes_client))
        .manage(Mutex::new(price_client))
        .manage(Mutex::new(charts_client))
        .manage(Mutex::new(config_client))
        .manage(Mutex::new(name_client))
        .manage(Mutex::new(devices_client))
        .manage(Mutex::new(assets_client))
        .manage(Mutex::new(search_client))
        .manage(Mutex::new(transactions_client))
        .manage(Mutex::new(metrics_client))
        .manage(Mutex::new(scan_client))
        .manage(Mutex::new(swap_client))
        .manage(Mutex::new(nft_client))
        .manage(Mutex::new(price_alert_client))
        .manage(Mutex::new(chain_client))
        .manage(Mutex::new(markets_client))
        .manage(Mutex::new(webhooks_client))
        .manage(Mutex::new(fiat_ip_check_client))
        .manage(Mutex::new(rewards_client))
        .manage(Mutex::new(redemption_client))
        .manage(Mutex::new(wallets_client))
        .manage(Mutex::new(notifications_client))
        .manage(auth_client);

    mount_routes(rocket, &settings.metrics.path)
}

async fn rocket_ws_prices(settings: Settings) -> Rocket<Build> {
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let price_client = PriceClient::new(database, cacher_client);
    let price_observer_config = PriceObserverConfig {
        redis_url: settings.redis.url.clone(),
    };

    rocket::build()
        .manage(Arc::new(Mutex::new(price_client)))
        .manage(Arc::new(Mutex::new(price_observer_config)))
        .mount("/", routes![websocket_prices::ws_health])
        .mount("/v1/ws", routes![websocket_prices::ws_prices])
        .register("/", catchers![catchers::default_catcher])
}

async fn rocket_ws_stream(settings: Settings) -> Rocket<Build> {
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let price_client = PriceClient::new(database.clone(), cacher_client);
    let stream_observer_config = websocket_stream::StreamObserverConfig {
        redis_url: settings.redis.url.clone(),
    };

    let jwt_config = devices::auth_config::JwtConfig {
        secret: settings.api.auth.jwt.secret.clone(),
        expiry: settings.api.auth.jwt.expiry,
    };
    let auth_config = devices::auth_config::AuthConfig::new(settings.api.auth.enabled, settings.api.auth.tolerance, jwt_config);

    rocket::build()
        .manage(auth_config)
        .manage(database)
        .manage(Arc::new(Mutex::new(price_client)))
        .manage(Arc::new(Mutex::new(stream_observer_config)))
        .mount("/v2/devices", routes![websocket_stream::ws_stream])
        .mount("/", routes![websocket_stream::ws_health])
        .register("/", catchers![catchers::default_catcher])
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();

    let sentry_config = settings.sentry.as_ref().map(|s| SentryConfig {
        dsn: s.dsn.clone(),
        sample_rate: s.sample_rate,
    });
    let _tracing = SentryTracing::init(sentry_config.as_ref(), "api");
    let service = std::env::args().nth(1).unwrap_or_default();
    let service = APIService::from_str(service.as_str()).ok().unwrap_or(APIService::Api);

    println!("api start service: {}", service.as_ref());

    match service {
        APIService::WebsocketPrices => {
            let rocket_api = rocket_ws_prices(settings.clone()).await;
            rocket_api.launch().await.expect("Failed to launch Rocket");
        }
        APIService::WebsocketStream => {
            let rocket_api = rocket_ws_stream(settings.clone()).await;
            rocket_api.launch().await.expect("Failed to launch Rocket");
        }
        APIService::Api => {
            let rocket_api = rocket_api(settings.clone()).await;
            rocket_api.launch().await.expect("Failed to launch Rocket");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rocket::async_test]
    async fn test_no_route_collisions() {
        let rocket = mount_routes(rocket::build(), "/metrics");
        if let Err(e) = rocket.ignite().await {
            let error = format!("{:?}", e);
            assert!(!error.contains("Collisions"), "Route collisions detected: {error}");
        }
    }
}
