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
mod node;
mod node_client;
mod parser;
mod parser_client;
mod prices;
mod scan;
mod scan_client;
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
use fiat::client::Client as FiatClient;
use fiat::FiatProviderFactory;
use metrics_client::MetricsClient;
use name_resolver::client::Client as NameClient;
use name_resolver::NameProviderFactory;
use node_client::Client as NodeClient;
use parser_client::ParserClient;
use pricer::client::PriceClient;
use rocket::fairing::AdHoc;
use rocket::tokio::sync::Mutex;
use rocket::{Build, Rocket};
use scan_client::ScanClient;
use settings::Settings;
use storage::DatabaseClient;
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
    let price_client = PriceClient::new(redis_url, postgres_url, &settings.clickhouse.url);
    let node_client = NodeClient::new(database_client).await;
    let config_client = ConfigClient::new(postgres_url).await;
    let providers = NameProviderFactory::create_providers(settings_clone.clone());
    let name_client = NameClient::new(providers);

    let pusher_client = PusherClient::new(settings.pusher.url);
    let devices_client =
        DevicesClient::new(postgres_url, pusher_client, settings.pusher.ios.topic).await;
    let transactions_client = TransactionsClient::new(postgres_url).await;
    let subscriptions_client = SubscriptionsClient::new(postgres_url).await;
    let metrics_client = MetricsClient::new(postgres_url).await;
    let scan_client = ScanClient::new(postgres_url).await;
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
    };
    let swapper_client = Swapper::build(swapper_configuration);
    let swap_client = SwapClient::new(postgres_url, swapper_client).await;
    let providers = FiatProviderFactory::new_providers(settings_clone.clone());
    let fiat_client = FiatClient::new(postgres_url, providers).await;

    rocket::build()
        .attach(AdHoc::on_ignite(
            "Tokio Runtime Configuration",
            |rocket| async {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio runtime");
                rocket.manage(runtime)
            },
        ))
        .manage(Mutex::new(fiat_client))
        .manage(Mutex::new(price_client))
        .manage(Mutex::new(node_client))
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
        .mount("/", routes![status::get_status,])
        .mount(
            "/v1",
            routes![
                prices::get_asset_price,
                prices::get_assets_prices,
                charts::get_charts,
                fiat_quotes::get_fiat_quotes,
                fiat_quotes::get_fiat_assets,
                fiat_quotes::get_fiat_rates,
                node::get_nodes,
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
                scan::get_scan_address,
                parser::get_parser_block,
                parser::get_parser_block_finalize,
                parser::get_parser_block_number_latest,
                swap::get_swap_quote,
                swap::post_swap_quote,
                swap::get_swap_assets,
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
