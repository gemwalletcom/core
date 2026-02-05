mod price_alerts_sender;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::WorkerJob;
use crate::worker::plan::JobPlanBuilder;
use job_runner::{JobHandle, ShutdownReceiver};
use price_alerts_sender::PriceAlertSender;
use pricer::PriceAlertClient;
use std::error::Error;
use storage::ConfigCacher;
use streamer::{StreamProducer, StreamProducerConfig};

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let config = ConfigCacher::new(database.clone());
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "send_price_alerts").await?;

    JobPlanBuilder::with_config(WorkerService::Alerter, runtime.plan(shutdown_rx), &config)
        .job(WorkerJob::SendPriceAlerts, {
            let database = database.clone();
            let stream_producer = stream_producer.clone();
            move || {
                let database = database.clone();
                let stream_producer = stream_producer.clone();
                async move {
                    let price_alert_client = PriceAlertClient::new(database.clone());
                    PriceAlertSender::new(database, price_alert_client, stream_producer).run_observer().await
                }
            }
        })
        .finish()
}
