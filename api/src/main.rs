#[macro_use] 
extern crate rocket;
mod status;
mod prices;
mod fiat_quotes;
mod nodes;
mod node_client;
mod config;
mod config_client;
mod plausible_client;
mod name;
mod charts;

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
use plausible_client:: Client as PlausibleClient;
use storage::database::DatabaseClient as DatabaseClient;
use name_resolver::client::Client as NameClient;
use rocket::tokio::sync::Mutex;
use rocket_prometheus::PrometheusMetrics;

async fn rocket(settings: Settings) -> Rocket<Build> {
    let redis_url = settings.redis.url.as_str();
    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);    
    database_client.migrations();

    let price_client = PriceClient::new(redis_url, postgres_url).await.unwrap();
    let node_client = NodeClient::new(database_client).await;
    let config_client = ConfigClient::new(postgres_url).await;
    let name_client = NameClient::new(
        settings.name.ens.url, 
        settings.name.ud.url,
        settings.name.ud.key.secret,
        settings.name.sns.url,
        settings.name.ton.url,
        settings.name.tree.url,
    );
    let plausible_client = PlausibleClient::new(&settings.plausible.url);
    let request_client = FiatClient::request_client(settings.fiat.timeout);
    let transak = TransakClient::new(request_client.clone(), settings.transak.key.public);
    let moonpay = MoonPayClient::new( request_client.clone(),  settings.moonpay.key.public,  settings.moonpay.key.secret);
    let mercuryo = MercuryoClient::new(request_client.clone(), settings.mercuryo.key.public);
    let ramp = RampClient::new(request_client.clone(), settings.ramp.key.public);
    let fiat_client = FiatClient::new(
        postgres_url,
        transak,
        moonpay,
        mercuryo,
        ramp
    ).await;
    let prometheus = PrometheusMetrics::new();

    rocket::build()
        .attach(AdHoc::on_ignite("Tokio Runtime Configuration", |rocket| async {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime");
            rocket.manage(runtime)
        }))
        .attach(prometheus.clone())
        .manage(Mutex::new(fiat_client))
        .manage(Mutex::new(price_client))
        .manage(Mutex::new(node_client))
        .manage(Mutex::new(config_client))
        .manage(Mutex::new(plausible_client))
        .manage(Mutex::new(name_client))
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
            nodes::get_nodes,
            config::get_config,
            name::get_name_resolve,
        ])
        .mount(settings.metrics.path, prometheus)
}

#[tokio::main]
async fn main() {

    let settings = Settings::new().unwrap();

    let rocket = rocket(settings).await;
    rocket.launch().await.expect("Failed to launch Rocket");
}