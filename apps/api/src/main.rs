#[macro_use]
extern crate rocket;
mod assets;
mod config;
mod devices;
mod fiat;
mod markets;
mod metrics;
mod model;
mod name;
mod nft;
mod parser;
mod price_alerts;
mod prices;
mod scan;
mod status;
mod subscriptions;
mod swap;
mod transactions;
mod websocket_prices;

use std::str::FromStr;
use std::sync::Arc;

use api_connector::PusherClient;
use assets::{AssetsChainProvider, AssetsClient, AssetsSearchClient};
use config::ConfigClient;
use devices::DevicesClient;
use fiat::{FiatClient, FiatProviderFactory};
use metrics::MetricsClient;
use model::APIService;
use name_resolver::client::Client as NameClient;
use name_resolver::NameProviderFactory;
use nft::NFTClient;
use parser::ParserClient;
use pricer::{ChartClient, MarketsClient, PriceAlertClient, PriceClient};
use rocket::fairing::AdHoc;
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket};
use scan::{ScanClient, ScanProviderFactory};
use search_index::SearchIndexClient;
use settings::Settings;
use settings_chain::ProviderFactory;
use storage::ClickhouseClient;
use subscriptions::SubscriptionsClient;
use swap::SwapClient;
use transactions::TransactionsClient;

async fn rocket_api(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let settings_clone = settings.clone();
    let price_client = PriceClient::new(redis_url, postgres_url);
    let clickhouse_client = ClickhouseClient::new(&settings_clone.clickhouse.url, &settings_clone.clickhouse.database);
    let charts_client = ChartClient::new(postgres_url, clickhouse_client);
    let config_client = ConfigClient::new(postgres_url).await;
    let price_alert_client = PriceAlertClient::new(postgres_url).await;
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers);

    let providers = ProviderFactory::new_providers(&settings);
    let assets_chain_provider = AssetsChainProvider::new(providers);

    let pusher_client = PusherClient::new(settings.pusher.url, settings.pusher.ios.topic);
    let devices_client = DevicesClient::new(postgres_url, pusher_client).await;
    let transactions_client = TransactionsClient::new(postgres_url).await;
    let subscriptions_client = SubscriptionsClient::new(postgres_url).await;
    let metrics_client = MetricsClient::new(postgres_url).await;

    let security_providers = ScanProviderFactory::create_providers(&settings_clone);
    let scan_client = ScanClient::new(postgres_url, security_providers).await;
    let parser_client = ParserClient::new(settings_clone.clone()).await;
    let assets_client = AssetsClient::new(postgres_url).await;
    let search_index_client = SearchIndexClient::new(&settings_clone.meilisearch.url.clone(), &settings_clone.meilisearch.key.clone());
    let assets_search_client = AssetsSearchClient::new(&search_index_client).await;
    let swap_client = SwapClient::new(postgres_url).await;
    let providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let ip_check_client = FiatProviderFactory::new_ip_check_client(settings_clone.clone());
    let fiat_client = FiatClient::new(postgres_url, redis_url, providers, ip_check_client).await;
    let nft_client = NFTClient::new(
        postgres_url,
        &settings.nft.nftscan.key.secret,
        &settings.nft.opensea.key.secret,
        &settings.nft.magiceden.key.secret,
    )
    .await;
    let markets_client = MarketsClient::new(postgres_url, redis_url);

    rocket::build()
        .attach(AdHoc::on_ignite("Tokio Runtime Configuration", |rocket| async {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            rocket.manage(runtime)
        }))
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
        .manage(Mutex::new(parser_client))
        .manage(Mutex::new(swap_client))
        .manage(Mutex::new(nft_client))
        .manage(Mutex::new(price_alert_client))
        .manage(Mutex::new(assets_chain_provider))
        .manage(Mutex::new(markets_client))
        .mount("/", routes![status::get_status])
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
                config::get_config,
                name::get_name_resolve,
                devices::add_device,
                devices::get_device,
                devices::update_device,
                devices::delete_device,
                devices::send_push_notification_device,
                assets::get_asset,
                assets::add_asset,
                assets::get_assets,
                assets::get_assets_list,
                assets::get_assets_search,
                assets::get_assets_by_device_id,
                assets::get_assets_ids_by_device_id,
                subscriptions::add_subscriptions,
                subscriptions::get_subscriptions,
                subscriptions::delete_subscriptions,
                transactions::get_transactions_by_device_id_old,
                transactions::get_transactions_by_device_id,
                transactions::get_transactions_by_hash,
                parser::get_parser_block,
                parser::get_parser_block_finalize,
                parser::get_parser_block_number_latest,
                swap::get_swap_assets,
                nft::get_nft_assets,
                nft::get_nft_assets_by_chain,
                nft::get_nft_collection,
                nft::get_nft_asset,
                nft::update_nft_collection,
                nft::update_nft_asset,
                price_alerts::get_price_alerts,
                price_alerts::add_price_alerts,
                price_alerts::delete_price_alerts,
                scan::scan_transaction,
                markets::get_markets,
            ],
        )
        .mount(settings.metrics.path, routes![metrics::get_metrics])
}

async fn rocket_ws_prices(settings: Settings) -> Rocket<Build> {
    let price_client = PriceClient::new(settings.redis.url.as_str(), settings.postgres.url.as_str());
    let price_client_arc = Arc::new(Mutex::new(price_client));

    rocket::build()
        .attach(AdHoc::on_ignite("Manage Price Client", |rocket| async move { rocket.manage(price_client_arc) }))
        .mount("/v1/ws", routes![websocket_prices::ws_prices])
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();

    let service = std::env::args().nth(1).unwrap_or_default();
    let service = APIService::from_str(service.as_str()).ok().unwrap_or(APIService::Api);

    println!("api start service: {}", service.as_ref());

    let rocket_api = match service {
        APIService::WebsocketPrices => rocket_ws_prices(settings).await,
        APIService::Api => rocket_api(settings).await,
    };
    rocket_api.launch().await.expect("Failed to launch Rocket");
}
