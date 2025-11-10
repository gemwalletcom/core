mod device_updater;
mod observers;
use cacher::CacherClient;
use device_updater::DeviceUpdater;
use job_runner::run_job;
use observers::InactiveDevicesObserver;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use streamer::StreamProducer;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let cacher_client = CacherClient::new(settings.redis.url.as_str()).await;

    let device_updater = run_job("device updater", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut device_updater = DeviceUpdater::new(&settings.postgres.url);
            async move { device_updater.update().await }
        }
    });

    let inactive_devices_observer = run_job("inactive devices observer", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "inactive_devices_observer").await.unwrap();
        move || {
            let mut observer = InactiveDevicesObserver::new(&settings.postgres.url, cacher_client.clone(), stream_producer.clone());
            async move { observer.observe().await }
        }
    });

    vec![Box::pin(device_updater), Box::pin(inactive_devices_observer)]
}
