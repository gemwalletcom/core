mod alerter;
mod assets;
mod device;
mod fiat;
mod model;
mod nft;
mod notifications;
mod pricer;
mod scan;
mod search;
mod support;
mod transaction;
mod version;

use crate::model::DaemonService;
use gem_tracing::{info_with_fields, SentryConfig, SentryTracing};
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;

#[tokio::main]
pub async fn main() {
    let service_arg = std::env::args().nth(1).unwrap_or_default();

    let service = DaemonService::from_str(service_arg.as_str()).unwrap_or_else(|_| {
        panic!(
            "Expected a valid service: {:?}",
            DaemonService::all().iter().map(|x| x.as_ref()).collect::<Vec<_>>()
        );
    });

    let settings = settings::Settings::new().unwrap();
    let sentry_config = settings.sentry.as_ref().map(|s| SentryConfig {
        dsn: s.dsn.clone(),
        sample_rate: s.sample_rate,
    });
    let _tracing = SentryTracing::init(sentry_config.as_ref(), service.as_ref());

    info_with_fields!("daemon start", service = service.as_ref());

    let services: Vec<Pin<Box<dyn Future<Output = ()> + Send>>> = match service {
        DaemonService::Alerter => alerter::jobs(settings.clone()).await,
        DaemonService::Pricer => pricer::jobs(settings.clone()).await,
        DaemonService::Fiat => fiat::jobs(settings.clone()).await,
        DaemonService::FiatConsumer => fiat::jobs_consumer(settings.clone()).await,
        DaemonService::Assets => assets::jobs(settings.clone()).await,
        DaemonService::Version => version::jobs(settings.clone()).await,
        DaemonService::Transaction => transaction::jobs(settings.clone()).await,
        DaemonService::Device => device::jobs(settings.clone()).await,
        DaemonService::Search => search::jobs(settings.clone()).await,
        DaemonService::Nft => nft::jobs(settings.clone()).await,
        DaemonService::Notifications => notifications::jobs(settings.clone()).await,
        DaemonService::Scan => scan::jobs(settings.clone()).await,
        DaemonService::ConsumerSupport => support::jobs(settings.clone()).await,
    };

    let _ = futures::future::join_all(services).await;
}
