mod assets;
mod chain;
mod config;
mod devices;
mod fiat;
mod markets;
mod metrics;
mod model;
mod name;
mod nft;
mod params;
mod price_alerts;
mod prices;
mod responders;
mod scan;
mod status;
mod subscriptions;
mod support;
mod swap;
mod transactions;
mod webhooks;
mod websocket_prices;

use std::str::FromStr;
use std::sync::Arc;

use ::nft::{NFTClient, NFTProviderConfig};
use api_connector::PusherClient;
use assets::{AssetsClient, AssetsSearchClient};
use cacher::CacherClient;
use config::ConfigClient;
use devices::DevicesClient;
use fiat::{FiatClient, FiatProviderFactory};
use gem_tracing::{SentryConfig, SentryTracing};
use metrics::MetricsClient;
use model::APIService;
use name_resolver::NameProviderFactory;
use name_resolver::client::Client as NameClient;
use pricer::{ChartClient, MarketsClient, PriceAlertClient, PriceClient};
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket, routes};
use scan::{ScanClient, ScanProviderFactory};
use search_index::SearchIndexClient;
use settings::Settings;
use settings_chain::{ChainProviders, ProviderFactory};
use streamer::StreamProducer;
use subscriptions::SubscriptionsClient;
use support::SupportClient;
use swap::SwapClient;
use transactions::TransactionsClient;
use webhooks::WebhooksClient;
use websocket_prices::PriceObserverConfig;

async fn rocket_api(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let settings_clone = settings.clone();
    let cacher_client = CacherClient::new(redis_url);
    let price_client = PriceClient::new(cacher_client.clone(), postgres_url);
    let charts_client = ChartClient::new(postgres_url);
    let config_client = ConfigClient::new(postgres_url).await;
    let price_alert_client = PriceAlertClient::new(postgres_url).await;
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers);

    let chain_client = chain::ChainClient::new(ChainProviders::new(ProviderFactory::new_providers(&settings)));

    let pusher_client = PusherClient::new(settings.pusher.url, settings.pusher.ios.topic);
    let devices_client = DevicesClient::new(postgres_url, pusher_client).await;
    let transactions_client = TransactionsClient::new(postgres_url).await;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "api").await.unwrap();
    let subscriptions_client = SubscriptionsClient::new(postgres_url, stream_producer.clone()).await;
    let metrics_client = MetricsClient::new(postgres_url).await;

    let security_providers = ScanProviderFactory::create_providers(&settings_clone);
    let scan_client = ScanClient::new(postgres_url, security_providers).await;
    let assets_client = AssetsClient::new(postgres_url).await;
    let search_index_client = SearchIndexClient::new(&settings_clone.meilisearch.url.clone(), &settings_clone.meilisearch.key.clone());
    let assets_search_client = AssetsSearchClient::new(&search_index_client).await;
    let swap_client = SwapClient::new(postgres_url).await;
    let providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let ip_check_client = FiatProviderFactory::new_ip_check_client(settings_clone.clone());
    let fiat_client = FiatClient::new(postgres_url, cacher_client.clone(), providers, ip_check_client, stream_producer.clone()).await;
    let nft_config = NFTProviderConfig::new(settings.nft.opensea.key.secret.clone(), settings.nft.magiceden.key.secret.clone());
    let nft_client = NFTClient::new(postgres_url, nft_config).await;
    let markets_client = MarketsClient::new(postgres_url, cacher_client);
    let webhooks_client = WebhooksClient::new(stream_producer.clone()).await;
    let support_client = SupportClient::new(postgres_url);

    rocket::build()
        .manage(Mutex::new(fiat_client))
        .manage(Mutex::new(price_client))
        .manage(Mutex::new(charts_client))
        .manage(Mutex::new(config_client))
        .manage(Mutex::new(name_client))
        .manage(Mutex::new(devices_client))
        .manage(Mutex::new(assets_client))
        .manage(Mutex::new(assets_search_client))
        .manage(Mutex::new(subscriptions_client))
        .manage(Mutex::new(transactions_client))
        .manage(Mutex::new(metrics_client))
        .manage(Mutex::new(scan_client))
        .manage(Mutex::new(swap_client))
        .manage(Mutex::new(nft_client))
        .manage(Mutex::new(price_alert_client))
        .manage(Mutex::new(chain_client))
        .manage(Mutex::new(markets_client))
        .manage(Mutex::new(webhooks_client))
        .manage(Mutex::new(support_client))
        .mount("/", routes![status::get_status, status::get_health])
        .mount(
            "/v1",
            routes![
                prices::get_price,
                prices::get_assets_prices,
                prices::get_charts,
                prices::get_fiat_rates,
                fiat::get_fiat_quotes,
                fiat::get_fiat_on_ramp_quotes,
                fiat::get_fiat_on_ramp_assets,
                fiat::get_fiat_off_ramp_assets,
                fiat::create_fiat_webhook,
                fiat::get_fiat_order,
                config::get_config,
                name::get_name_resolve,
                devices::add_device,
                devices::get_device,
                devices::update_device,
                devices::delete_device,
                devices::send_push_notification_device,
                assets::get_asset,
                assets::get_assets,
                assets::add_asset,
                assets::get_assets_search,
                assets::get_assets_by_device_id,
                subscriptions::add_subscriptions,
                subscriptions::get_subscriptions,
                subscriptions::delete_subscriptions,
                transactions::get_transactions_by_device_id_v1,
                transactions::get_transactions_by_id,
                chain::transaction::get_latest_block_number,
                chain::transaction::get_block_transactions,
                chain::transaction::get_block_transactions_finalize,
                swap::get_swap_assets,
                nft::get_nft_assets_old,
                nft::get_nft_assets_by_chain,
                nft::get_nft_collection,
                nft::get_nft_asset,
                nft::get_nft_asset_image_preview,
                nft::update_nft_collection,
                nft::update_nft_asset,
                price_alerts::get_price_alerts,
                price_alerts::add_price_alerts,
                price_alerts::delete_price_alerts,
                scan::scan_transaction,
                scan::get_scan_address,
                markets::get_markets,
                chain::staking::get_validators,
                chain::staking::get_staking_apy,
                chain::token::get_token,
                chain::balance::get_balances_coin,
                chain::balance::get_balances_assets,
                chain::balance::get_balances_staking,
                chain::transaction::get_transactions,
                webhooks::create_support_webhook,
                support::add_device,
                support::get_support_device,
            ],
        )
        .mount("/v2", routes![transactions::get_transactions_by_device_id_v2, nft::get_nft_assets_v2,])
        .mount(settings.metrics.path, routes![metrics::get_metrics])
}

async fn rocket_ws_prices(settings: Settings) -> Rocket<Build> {
    let cacher_client = CacherClient::new(&settings.redis.url);
    let price_client = PriceClient::new(cacher_client, settings.postgres.url.as_str());
    let price_observer_config = PriceObserverConfig { redis_url: settings.redis.url };
    rocket::build()
        .manage(Arc::new(Mutex::new(price_client)))
        .manage(Arc::new(Mutex::new(price_observer_config)))
        .mount("/v1/ws", routes![websocket_prices::ws_prices])
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

    let rocket_api = match service {
        APIService::WebsocketPrices => rocket_ws_prices(settings.clone()).await,
        APIService::Api => rocket_api(settings.clone()).await,
    };

    rocket_api.launch().await.expect("Failed to launch Rocket");
}
