#[macro_use]
extern crate rocket;
mod asset;
mod asset_client;
mod charts;
mod config;
mod config_client;
mod device;
mod device_client;
mod fiat_quotes;
mod metrics;
mod metrics_client;
mod name;
mod nft;
mod nft_client;
mod parser;
mod parser_client;
mod price_alerts;
mod prices;
mod response;
mod security_providers;
mod security_scan;
mod status;
mod subscription;
mod subscription_client;
mod swap;
mod swap_client;
mod transaction;
mod transaction_client;

use api_connector::PusherClient;
use asset_client::AssetsClient;
use config_client::Client as ConfigClient;
use device_client::DevicesClient;
use fiat::client::Client as FiatProvider;
use fiat::FiatProviderFactory;
use metrics_client::MetricsClient;
use name_resolver::client::Client as NameClient;
use name_resolver::NameProviderFactory;
use nft_client::NFTClient;
use parser_client::ParserClient;
use price_alert::PriceAlertClient;
use pricer::chart_client::ChartClient;
use pricer::price_client::PriceClient;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket};
use security_providers::SecurityProviderFactory;
use security_scan::SecurityScanClient;
use settings::{Settings};
use storage::{ClickhouseClient, DatabaseClient};
use subscription_client::SubscriptionsClient;
use swap_client::SwapClient;
use swapper::{Swapper, SwapperClientConfiguration, SwapperConfiguration};
use transaction_client::TransactionsClient;

async fn rocket(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);
    database_client.migrations();

    let settings_clone = settings.clone();
    let price_client = PriceClient::new(redis_url, postgres_url);
    let clickhouse_client = ClickhouseClient::new(&settings_clone.clickhouse.url, &settings_clone.clickhouse.database);
    let charts_client = ChartClient::new(postgres_url, clickhouse_client);
    let config_client = ConfigClient::new(postgres_url).await;
    let price_alert_client = PriceAlertClient::new(postgres_url).await;
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers);

    let pusher_client = PusherClient::new(settings.pusher.url);
    let devices_client = DevicesClient::new(postgres_url, pusher_client, settings.pusher.ios.topic).await;
    let transactions_client = TransactionsClient::new(postgres_url).await;
    let subscriptions_client = SubscriptionsClient::new(postgres_url).await;
    let metrics_client = MetricsClient::new(postgres_url).await;

    let security_providers = SecurityProviderFactory::create_providers(&settings_clone);
    let scan_client = SecurityScanClient::new(postgres_url, security_providers).await;
    let parser_client = ParserClient::new(settings_clone.clone()).await;
    let assets_client = AssetsClient::new(postgres_url).await;
    let swapper_configuration = SwapperConfiguration {
        oneinch: SwapperClientConfiguration {
            url: settings.swap.oneinch.url,
            key: settings.swap.oneinch.key,
            fee_percent: settings.swap.oneinch.fee.percent,
            fee_address: settings.swap.oneinch.fee.address,
        },
        jupiter: SwapperClientConfiguration {
            url: settings.swap.jupiter.url,
            key: "".to_string(),
            fee_percent: settings.swap.jupiter.fee.percent,
            fee_address: settings.swap.jupiter.fee.address,
        },
        thorchain: SwapperClientConfiguration {
            url: settings.swap.thorchain.url,
            key: "".to_string(),
            fee_percent: settings.swap.thorchain.fee.percent,
            fee_address: settings.swap.thorchain.fee.address,
        },
        aftermath: SwapperClientConfiguration {
            url: settings.swap.aftermath.url,
            key: "".to_string(),
            fee_percent: settings.swap.aftermath.fee.percent,
            fee_address: settings.swap.aftermath.fee.address,
        },
    };
    let swapper_client = Swapper::build(swapper_configuration);
    let swap_client = SwapClient::new(postgres_url, swapper_client).await;
    let providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let fiat_client = FiatProvider::new(postgres_url, providers).await;
    let nft_client = NFTClient::new(postgres_url).await;

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
        .manage(Mutex::new(subscriptions_client))
        .manage(Mutex::new(transactions_client))
        .manage(Mutex::new(metrics_client))
        .manage(Mutex::new(scan_client))
        .manage(Mutex::new(parser_client))
        .manage(Mutex::new(swap_client))
        .manage(Mutex::new(nft_client))
        .manage(Mutex::new(price_alert_client))
        .mount("/", routes![status::get_status,])
        .mount(
            "/v1",
            routes![
                prices::get_price,
                prices::get_assets_prices,
                charts::get_charts,
                fiat_quotes::get_fiat_quotes,
                fiat_quotes::get_fiat_assets,
                fiat_quotes::get_fiat_on_ramp_quotes,
                fiat_quotes::get_fiat_on_ramp_assets,
                fiat_quotes::create_fiat_webhook,
                config::get_config,
                name::get_name_resolve,
                device::add_device,
                device::get_device,
                device::update_device,
                device::delete_device,
                device::send_push_notification_device,
                asset::get_asset,
                asset::get_assets,
                asset::get_assets_list,
                asset::get_assets_search,
                asset::get_assets_ids_by_device_id,
                subscription::add_subscriptions,
                subscription::get_subscriptions,
                subscription::delete_subscriptions,
                transaction::get_transactions_by_device_id,
                transaction::get_transactions_by_hash,
                parser::get_parser_block,
                parser::get_parser_block_finalize,
                parser::get_parser_block_number_latest,
                swap::post_swap_quote,
                swap::get_swap_assets,
                nft::get_nft_collections,
                nft::get_nft_collectibles,
                nft::get_nft_collections_by_chain_address,
                nft::get_nft_collectibles_by_chain_address,
                price_alerts::get_price_alerts,
                price_alerts::add_price_alerts,
                price_alerts::delete_price_alerts,
                security_scan::scan,
            ],
        )
        .mount(settings.metrics.path, routes![metrics::get_metrics,])
}

#[tokio::main]
async fn main() {
    let settings = Settings::new().unwrap();

    let rocket = rocket(settings).await;
    rocket.launch().await.expect("Failed to launch Rocket");
}
