mod consumers;
mod model;
mod parser;
mod pusher;
mod setup;
mod worker;

use crate::model::{ConsumerService, DaemonService, WorkerService};
use gem_tracing::{SentryConfig, SentryTracing, info_with_fields};
use std::str::FromStr;

#[tokio::main]
pub async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let service_arg = args.iter().skip(1).map(|s| s.as_str()).collect::<Vec<_>>().join(" ");

    let service = DaemonService::from_str(&service_arg).unwrap_or_else(|e| {
        panic!(
            "{}\nUsage examples: \n daemon parser \n daemon parser ethereum \n daemon worker alerter \n daemon consumer fetch_transactions",
            e
        );
    });

    let settings = settings::Settings::new().unwrap();
    let sentry_config = settings.sentry.as_ref().map(|s| SentryConfig {
        dsn: s.dsn.clone(),
        sample_rate: s.sample_rate,
    });
    let _tracing = SentryTracing::init(sentry_config.as_ref(), service.as_ref());

    info_with_fields!("daemon start", service = service.name());

    match service {
        DaemonService::Setup => {
            let _ = setup::run_setup(settings).await;
        }
        DaemonService::SetupDev => {
            let _ = setup::run_setup_dev(settings).await;
        }
        DaemonService::Worker(service) => {
            run_worker_mode(settings, service).await;
        }
        DaemonService::Parser(chain) => {
            parser::run(settings, chain).await.expect("Parser failed");
        }
        DaemonService::Consumer(service) => {
            run_consumer_mode(settings, service).await.expect("Consumer failed");
        }
    }
}

async fn run_worker_mode(settings: settings::Settings, service: WorkerService) {
    let services = match service {
        WorkerService::Alerter => worker::alerter::jobs(settings).await.unwrap(),
        WorkerService::Pricer => worker::pricer::jobs(settings).await.unwrap(),
        WorkerService::PricesDex => worker::prices_dex::jobs(settings).await,
        WorkerService::Fiat => worker::fiat::jobs(settings).await.unwrap(),
        WorkerService::Assets => worker::assets::jobs(settings).await.unwrap(),
        WorkerService::Version => worker::version::jobs(settings).await.unwrap(),
        WorkerService::Transaction => worker::transaction::jobs(settings).await.unwrap(),
        WorkerService::Device => worker::device::jobs(settings).await.unwrap(),
        WorkerService::Search => worker::search::jobs(settings).await.unwrap(),
        WorkerService::Nft => worker::nft::jobs(settings).await,
        WorkerService::Scan => worker::scan::jobs(settings).await.unwrap(),
        WorkerService::Rewards => worker::rewards::jobs(settings).await.unwrap(),
    };
    let _ = futures::future::join_all(services).await;
}

async fn run_consumer_mode(settings: settings::Settings, service: ConsumerService) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match service {
        ConsumerService::FetchAddressTransactions => consumers::run_consumer_fetch_address_transactions(settings).await,
        ConsumerService::StoreTransactions => consumers::run_consumer_store_transactions(settings).await,
        ConsumerService::FetchBlocks => consumers::run_consumer_fetch_blocks(settings).await,
        ConsumerService::FetchAssets => consumers::run_consumer_fetch_assets(settings).await,
        ConsumerService::FetchTokenAssociations => consumers::run_consumer_fetch_token_associations(settings).await,
        ConsumerService::FetchCoinAssociations => consumers::run_consumer_fetch_coin_associations(settings).await,
        ConsumerService::StoreAssetsAssociations => consumers::run_consumer_store_assets_associations(settings).await,
        ConsumerService::FetchNftAssociations => consumers::run_consumer_fetch_nft_associations(settings).await,
        ConsumerService::Notifications => consumers::notifications::run(settings).await,
        ConsumerService::InAppNotifications => consumers::run_consumer_in_app_notifications(settings).await,
        ConsumerService::Rewards => consumers::run_consumer_rewards(settings).await,
        ConsumerService::RewardsRedemptions => consumers::run_rewards_redemption_consumer(settings).await,
        ConsumerService::Support => consumers::run_consumer_support(settings).await,
        ConsumerService::Fiat => consumers::run_consumer_fiat(settings).await,
        ConsumerService::StorePrices => consumers::run_consumer_store_prices(settings).await,
        ConsumerService::StoreCharts => consumers::run_consumer_store_charts(settings).await,
    }
}
