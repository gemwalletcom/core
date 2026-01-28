mod device_updater;
mod observers;

use cacher::CacherClient;
use device_updater::DeviceUpdater;
use job_runner::{ShutdownReceiver, run_job};
use observers::InactiveDevicesObserver;
use primitives::ConfigKey;
use settings::Settings;
use std::error::Error;
use std::sync::Arc;
use storage::ConfigCacher;
use streamer::StreamProducer;
use tokio::task::JoinHandle;

pub async fn jobs(settings: Settings, shutdown_rx: ShutdownReceiver) -> Result<Vec<JoinHandle<()>>, Box<dyn Error + Send + Sync>> {
    let database = storage::Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let cacher_client = CacherClient::new(settings.redis.url.as_str()).await;

    let device_updater = tokio::spawn(run_job("device updater", config.get_duration(ConfigKey::DeviceTimerUpdater)?, shutdown_rx.clone(), {
        let database = database.clone();
        move || {
            let device_updater = DeviceUpdater::new(database.clone());
            async move { device_updater.update().await }
        }
    }));

    let inactive_devices_observer = tokio::spawn(run_job(
        "inactive devices observer",
        config.get_duration(ConfigKey::DeviceTimerInactiveObserver)?,
        shutdown_rx,
        {
            let settings = Arc::new(settings.clone());
            let database = database.clone();
            let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "inactive_devices_observer").await.unwrap();
            move || {
                let observer = InactiveDevicesObserver::new(database.clone(), cacher_client.clone(), stream_producer.clone());
                async move { observer.observe().await }
            }
        },
    ));

    Ok(vec![device_updater, inactive_devices_observer])
}
