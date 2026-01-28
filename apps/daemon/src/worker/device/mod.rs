mod device_updater;
mod observers;

use cacher::CacherClient;
use device_updater::DeviceUpdater;
use job_runner::{JobStatusReporter, ShutdownReceiver, run_job};
use observers::InactiveDevicesObserver;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, reporter: Arc<dyn JobStatusReporter>, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let cacher_client = CacherClient::new(settings.redis.url.as_str()).await;

    let device_updater = tokio::spawn(run_job(
        "update_devices",
        config.get_duration(ConfigKey::DeviceTimerUpdater)?,
        reporter.clone(),
        shutdown_rx.clone(),
        {
            let database = database.clone();
            move || {
                let device_updater = DeviceUpdater::new(database.clone());
                async move { device_updater.update().await }
            }
        },
    ));

    let inactive_devices_observer = tokio::spawn(run_job(
        "observe_inactive_devices",
        config.get_duration(ConfigKey::DeviceTimerInactiveObserver)?,
        reporter.clone(),
        shutdown_rx,
        {
            let settings = Arc::new(settings.clone());
            let database = database.clone();
            let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
            let stream_producer = StreamProducer::new(&rabbitmq_config, "observe_inactive_devices").await.unwrap();
            move || {
                let observer = InactiveDevicesObserver::new(database.clone(), cacher_client.clone(), stream_producer.clone());
                async move { observer.observe().await }
            }
        },
    ));

    Ok(vec![device_updater, inactive_devices_observer])
}
