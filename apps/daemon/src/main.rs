mod alerter;
mod assets;
mod device;
mod fiat;
mod model;
mod pricer;
mod transaction;
mod version;

use crate::model::DaemonService;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

#[tokio::main]
pub async fn main() {
    println!("daemon init");

    let service = std::env::args().nth(1).unwrap_or_default();

    println!("daemon start service: {service}");

    let settings = settings::Settings::new().unwrap();

    let service = DaemonService::from_str(service.as_str()).expect("Expected a valid service");

    // Pin the futures when creating the services vector
    let services: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = match service {
        DaemonService::Alerter => alerter::jobs(settings.clone()).await,
        DaemonService::Pricer => pricer::jobs(settings.clone()).await,
        DaemonService::Fiat => fiat::jobs(settings.clone()).await,
        DaemonService::Assets => assets::jobs(settings.clone()).await,
        DaemonService::Version => version::jobs(settings.clone()).await,
        DaemonService::Transaction => transaction::jobs(settings.clone()).await,
        DaemonService::Device => device::jobs(settings.clone()).await,
    };

    let _ = futures::future::join_all(services).await;
}
