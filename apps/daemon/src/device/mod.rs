mod device_updater;
use device_updater::DeviceUpdater;
use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

pub async fn jobs(settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    let device_updater = run_job("device updater", Duration::from_secs(86400), {
        let settings = Arc::new(settings.clone());
        move || {
            let mut device_updater = DeviceUpdater::new(&settings.postgres.url);
            async move { device_updater.update().await }
        }
    });

    vec![Box::pin(device_updater)]
}
