use fiat::FiatProviderFactory;
use fiat_assets_updater::FiatAssetsUpdater;
use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use streamer::{run_consumer, ConsumerConfig, QueueName, StreamReader};

use crate::fiat::fiat_webhook_consumer::FiatWebhookConsumer;

mod fiat_assets_updater;
pub mod fiat_webhook_consumer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let update_fiat_assets_job = run_job("Update fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_fiat_assets("fiat_update_assets").await }
        }
    });

    let update_fiat_provider_countries_job = run_job("Update providers countries", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_fiat_countries("fiat_update_countries").await }
        }
    });

    let update_fiat_buyable_assets_job = run_job("Update fiat buyable/sellable assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_buyable_sellable_assets().await }
        }
    });

    let update_trending_fiat_assets_job = run_job("Update trending fiat assets", Duration::from_secs(3600), {
        let settings = Arc::new(settings.clone());
        move || {
            let providers = FiatProviderFactory::new_providers((*settings).clone());
            let mut fiat_assets_updater = FiatAssetsUpdater::new(&settings.postgres.url, providers);
            async move { fiat_assets_updater.update_trending_fiat_assets().await }
        }
    });

    vec![
        Box::pin(update_fiat_assets_job),
        Box::pin(update_fiat_provider_countries_job),
        Box::pin(update_fiat_buyable_assets_job),
        Box::pin(update_trending_fiat_assets_job),
    ]
}

pub async fn jobs_consumer(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let fiat_webhook_consumer_job = job_runner::run_job("Fiat webhook consumer", Duration::from_secs(86000), {
        let settings = Arc::new(settings.clone());
        move || {
            let settings_clone = settings.clone();
            async move {
                let consumer = FiatWebhookConsumer::new(&settings_clone.postgres.url, (*settings_clone).clone());
                let stream_reader = StreamReader::new(&settings_clone.rabbitmq.url, "daemon_fiat_consumer").await.unwrap();
                let _ = run_consumer(
                    "fiat_webhook_consumer",
                    stream_reader,
                    QueueName::FiatOrderWebhooks,
                    consumer,
                    ConsumerConfig::default(),
                )
                .await;
            }
        }
    });

    vec![Box::pin(fiat_webhook_consumer_job)]
}
