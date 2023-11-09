#[macro_use] 
extern crate rocket;
mod status;
mod prices;
mod fiat_quotes;
mod node;
mod node_client;
mod config;
mod config_client;
mod name;
mod charts;
mod device;
mod device_client;
mod asset;
mod asset_client;
mod subscription;
mod subscription_client;
mod transaction;
mod transaction_client;
mod metrics;
mod metrics_client;
mod scan;
mod scan_client;
mod parser;
mod parser_client;
mod swap;
mod swap_client;

use api_connector::PusherClient;
use asset_client::AssetsClient;
use fiat::mercuryo::MercuryoClient;
use fiat::moonpay::MoonPayClient;
use fiat::transak::TransakClient;
use fiat::ramp::RampClient;
use node_client::Client as NodeClient;
use rocket::fairing::AdHoc;
use rocket::{Build, Rocket};
use settings::Settings;
use pricer::client::Client as PriceClient;
use fiat::client::Client as FiatClient;
use config_client::Client as ConfigClient;
use storage::DatabaseClient as DatabaseClient;
use name_resolver::client::Client as NameClient;
use device_client::DevicesClient;
use subscription_client::SubscriptionsClient;
use rocket::tokio::sync::Mutex;
use transaction_client::TransactionsClient;
use metrics_client::MetricsClient;
use scan_client::ScanClient;
use parser_client::ParserClient;
use swap_client::SwapClient;
use swapper::{SwapperClient, OneInchClient};

async fn rocket(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);    
    database_client.migrations();

    let settings_clone = settings.clone();
    let price_client = PriceClient::new(redis_url, postgres_url).await.unwrap();
    let node_client = NodeClient::new(database_client).await;
    let config_client = ConfigClient::new(postgres_url).await;
    let name_client = NameClient::new(
        settings.name.ens.url, 
        settings.name.ud.url,
        settings.name.ud.key.secret,
        settings.name.sns.url,
        settings.name.ton.url,
        settings.name.eths.url,
        settings.name.spaceid.url,
        settings.name.did.url,
        settings.name.suins.url,
    );
    
    let pusher_client = PusherClient::new(settings.pusher.url);
    let devices_client = DevicesClient::new(postgres_url, pusher_client, settings.pusher.ios.topic).await;
    let transactions_client = TransactionsClient::new(postgres_url).await;
    let subscriptions_client = SubscriptionsClient::new(postgres_url).await;
    let metrics_client = MetricsClient::new(postgres_url).await;
    let scan_client = ScanClient::new(postgres_url).await;
    let parser_client = ParserClient::new(settings_clone).await;
    let assets_client = AssetsClient::new(postgres_url).await;
    let oneinch_client = OneInchClient::new(settings.swap.oneinch.url, settings.swap.oneinch.key, settings.swap.fee, settings.swap.fee_address);
    let swapper_client = SwapperClient::new(oneinch_client);
    let swap_client = SwapClient::new( postgres_url, swapper_client).await;
    
    let request_client = FiatClient::request_client(settings.fiat.timeout);
    let transak = TransakClient::new(request_client.clone(), settings.transak.key.public);
    let moonpay = MoonPayClient::new( request_client.clone(),  settings.moonpay.key.public,  settings.moonpay.key.secret);
    let mercuryo = MercuryoClient::new(request_client.clone(),  settings.mercuryo.key.public, settings.mercuryo.key.secret);
    let ramp = RampClient::new(request_client.clone(), settings.ramp.key.public);
    let fiat_client = FiatClient::new(
        postgres_url,
        transak,
        moonpay,
        mercuryo,
        ramp
    ).await;

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
        .mount("/", routes![
            status::get_status,
        ])
        .mount("/v1", routes![
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
        ])
        .mount(settings.metrics.path, routes![
            metrics::get_metrics,
        ])
}

#[tokio::main]
async fn main() {

    let settings = Settings::new().unwrap();

    let rocket = rocket(settings).await;
    rocket.launch().await.expect("Failed to launch Rocket");
}