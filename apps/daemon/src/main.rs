mod consumers;
mod model;
mod parser;
mod pusher;
mod setup;
mod shutdown;
mod worker;

use crate::model::{ConsumerService, DaemonService, WorkerService};
use crate::shutdown::ShutdownReceiver;
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
    let (shutdown_tx, shutdown_rx) = shutdown::channel();
    let shutdown_timeout = settings.daemon.shutdown.timeout;

    let signal_handle = shutdown::spawn_signal_handler(shutdown_tx);

    let services = match service {
        WorkerService::Alerter => worker::alerter::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Pricer => worker::pricer::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::PricesDex => worker::prices_dex::jobs(settings, shutdown_rx).await,
        WorkerService::Fiat => worker::fiat::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Assets => worker::assets::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Version => worker::version::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Transaction => worker::transaction::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Device => worker::device::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Search => worker::search::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Nft => worker::nft::jobs(settings, shutdown_rx).await,
        WorkerService::Scan => worker::scan::jobs(settings, shutdown_rx).await.unwrap(),
        WorkerService::Rewards => worker::rewards::jobs(settings, shutdown_rx).await.unwrap(),
    };

    signal_handle.await.ok();
    shutdown::wait_with_timeout(services, shutdown_timeout).await;
    info_with_fields!("all workers stopped", status = "ok");
}

async fn run_consumer_mode(settings: settings::Settings, service: ConsumerService) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (shutdown_tx, shutdown_rx) = shutdown::channel();

    shutdown::spawn_signal_handler(shutdown_tx);

    let result = run_consumer(settings, service, shutdown_rx).await;

    info_with_fields!("consumer stopped", status = "ok");
    result
}

async fn run_consumer(settings: settings::Settings, service: ConsumerService, shutdown_rx: ShutdownReceiver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match service {
        ConsumerService::FetchAddressTransactions => consumers::run_consumer_fetch_address_transactions(settings, shutdown_rx).await,
        ConsumerService::StoreTransactions => consumers::run_consumer_store_transactions(settings, shutdown_rx).await,
        ConsumerService::FetchBlocks => consumers::run_consumer_fetch_blocks(settings, shutdown_rx).await,
        ConsumerService::FetchAssets => consumers::run_consumer_fetch_assets(settings, shutdown_rx).await,
        ConsumerService::FetchTokenAssociations => consumers::run_consumer_fetch_token_associations(settings, shutdown_rx).await,
        ConsumerService::FetchCoinAssociations => consumers::run_consumer_fetch_coin_associations(settings, shutdown_rx).await,
        ConsumerService::StoreAssetsAssociations => consumers::run_consumer_store_assets_associations(settings, shutdown_rx).await,
        ConsumerService::FetchNftAssociations => consumers::run_consumer_fetch_nft_associations(settings, shutdown_rx).await,
        ConsumerService::Notifications => consumers::notifications::run(settings, shutdown_rx).await,
        ConsumerService::InAppNotifications => consumers::run_consumer_in_app_notifications(settings, shutdown_rx).await,
        ConsumerService::Rewards => consumers::run_consumer_rewards(settings, shutdown_rx).await,
        ConsumerService::RewardsRedemptions => consumers::run_rewards_redemption_consumer(settings, shutdown_rx).await,
        ConsumerService::Support => consumers::run_consumer_support(settings, shutdown_rx).await,
        ConsumerService::Fiat => consumers::run_consumer_fiat(settings, shutdown_rx).await,
        ConsumerService::StorePrices => consumers::run_consumer_store_prices(settings, shutdown_rx).await,
        ConsumerService::StoreCharts => consumers::run_consumer_store_charts(settings, shutdown_rx).await,
        ConsumerService::FetchPrices => consumers::run_consumer_fetch_prices(settings, shutdown_rx).await,
    }
}
