mod charts_updater;
mod markets_updater;
mod observed_prices_updater;
pub mod prices_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::{JobVariant, WorkerJob};
use crate::worker::plan::JobPlanBuilder;
use chrono::{Duration, Utc};
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use cacher::CacherClient;
use charts_updater::ChartsUpdater;
use coingecko::CoinGeckoClient;
use gem_client::ReqwestClient;
use job_runner::{JobHandle, ShutdownReceiver};
use markets_updater::MarketsUpdater;
use observed_prices_updater::ObservedPricesUpdater;
use pricer::{MarketsClient, PriceClient};
use prices::{CoinGeckoPricesProvider, JupiterProvider, PriceAssetsProvider, PriceProvider, PythProvider};
use prices_updater::PricesUpdater;
use primitives::{ChartTimeframe, ConfigKey, ConfigParamKey};
use settings::Settings;
use storage::database::prices::PriceFilter;
use storage::repositories::prices_providers_repository::PricesProvidersRepository;
use storage::{ConfigCacher, Database, PricesRepository};
use streamer::{StreamProducer, StreamProducerConfig};

type Providers = Arc<HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>>;

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver, only: Option<PriceProvider>) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = Arc::new(ConfigCacher::new(database.clone()));
    let producer_assets = stream_producer(&settings, "prices_provider_assets").await?;
    let producer_prices = stream_producer(&settings, "prices_provider_prices").await?;
    let enabled_providers: Vec<PriceProvider> = database
        .prices_providers()?
        .get_prices_providers()?
        .into_iter()
        .filter(|p| p.enabled)
        .map(|p| p.id.0)
        .collect();
    let providers: Providers = Arc::new(enabled_providers.iter().map(|p| (*p, build_provider(*p, &settings))).collect());

    let builder = JobPlanBuilder::with_config(WorkerService::Prices, runtime.plan(shutdown_rx), config.as_ref());
    let builder = if only.is_none() {
        add_platform_jobs(builder, &database, &cacher_client, &config, &providers, producer_prices.clone())
    } else {
        builder
    };
    enabled_providers
        .into_iter()
        .filter(|p| only.is_none_or(|o| o == *p))
        .fold(builder, |builder, provider| {
            add_provider_jobs(
                builder,
                &database,
                &cacher_client,
                &settings,
                &config,
                provider,
                providers[&provider].clone(),
                &producer_assets,
                &producer_prices,
            )
        })
        .finish()
}

fn add_platform_jobs<'a>(
    builder: JobPlanBuilder<'a>,
    database: &Database,
    cacher_client: &CacherClient,
    config: &Arc<ConfigCacher>,
    providers: &Providers,
    producer: StreamProducer,
) -> JobPlanBuilder<'a> {
    builder
        .job(WorkerJob::CleanupOutdatedAssets, {
            let database = database.clone();
            let config = config.clone();
            move |_| {
                let database = database.clone();
                let config = config.clone();
                async move {
                    let cutoff = Utc::now() - Duration::seconds(config.get_duration(ConfigKey::PriceOutdated)?.as_secs() as i64);
                    let ids = database
                        .prices()?
                        .get_prices_by_filter(vec![PriceFilter::UpdatedBefore(cutoff.naive_utc())])?
                        .into_iter()
                        .map(|p| p.id)
                        .collect();
                    Ok::<usize, Box<dyn std::error::Error + Send + Sync>>(database.prices()?.delete_prices(ids)?)
                }
            }
        })
        .job(
            WorkerJob::AggregateHourlyCharts,
            charts_job(database, cacher_client, ChartsAction::Aggregate(ChartTimeframe::Hourly)),
        )
        .job(
            WorkerJob::AggregateDailyCharts,
            charts_job(database, cacher_client, ChartsAction::Aggregate(ChartTimeframe::Daily)),
        )
        .job(
            WorkerJob::CleanupChartsHourly,
            charts_job(database, cacher_client, ChartsAction::Cleanup(ChartTimeframe::Hourly)),
        )
        .job(WorkerJob::UpdateObservedPrices, {
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            let config = config.clone();
            let providers = providers.clone();
            let producer = producer.clone();
            move |_| {
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                let config = config.clone();
                let providers = providers.clone();
                let producer = producer.clone();
                async move {
                    let max_observed_assets = config.get_usize(ConfigKey::PriceObservedMaxAssets)?;
                    let min_observers = config.get_usize(ConfigKey::PriceObservedMinObservers)?;
                    ObservedPricesUpdater::new(cacher_client, database, providers, producer, max_observed_assets, min_observers)
                        .update()
                        .await
                }
            }
        })
}

fn add_provider_jobs<'a>(
    builder: JobPlanBuilder<'a>,
    database: &Database,
    cacher_client: &CacherClient,
    settings: &Settings,
    config: &ConfigCacher,
    provider: PriceProvider,
    provider_instance: Arc<dyn PriceAssetsProvider>,
    producer_assets: &StreamProducer,
    producer_prices: &StreamProducer,
) -> JobPlanBuilder<'a> {
    let slug = provider.id().to_string();
    let assets_variant = match config.get_param_duration(&ConfigParamKey::PriceProviderAssetsDuration(provider)) {
        Ok(duration) => JobVariant::labeled(WorkerJob::UpdatePricesAssets, slug.clone()).every(duration),
        Err(_) => JobVariant::labeled(WorkerJob::UpdatePricesAssets, slug.clone()),
    };
    let assets_new_variant = match config.get_param_duration(&ConfigParamKey::PriceProviderAssetsNewDuration(provider)) {
        Ok(duration) => JobVariant::labeled(WorkerJob::UpdatePricesAssetsNew, slug.clone()).every(duration),
        Err(_) => JobVariant::labeled(WorkerJob::UpdatePricesAssetsNew, slug.clone()),
    };
    let builder = builder
        .job(
            assets_variant,
            provider_job(database, provider_instance.clone(), producer_assets.clone(), |u| async move { u.update_assets().await }),
        )
        .job(
            assets_new_variant,
            provider_job(database, provider_instance.clone(), producer_assets.clone(), |u| async move { u.update_assets_new().await }),
        );
    match provider {
        PriceProvider::Coingecko => builder
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesTop, slug.clone()),
                provider_job(database, provider_instance.clone(), producer_prices.clone(), |u| async move {
                    u.update_prices_window(0, 500).await
                }),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesHigh, slug.clone()),
                provider_job(database, provider_instance.clone(), producer_prices.clone(), |u| async move {
                    u.update_prices_window(500, 2500).await
                }),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesLow, slug),
                provider_job(database, provider_instance, producer_prices.clone(), |u| async move {
                    u.update_prices_window(3000, usize::MAX).await
                }),
            )
            .job(WorkerJob::UpdateMarkets, {
                let coingecko = CoinGeckoClient::new(&settings.coingecko.key.secret);
                let markets_client = MarketsClient::new(database.clone(), cacher_client.clone());
                move |_| {
                    let updater = MarketsUpdater::new(markets_client.clone(), coingecko.clone());
                    Box::pin(async move { updater.update_markets().await })
                }
            }),
        PriceProvider::Pyth | PriceProvider::Jupiter => {
            let prices_variant = match config.get_param_duration(&ConfigParamKey::PriceProviderPricesDuration(provider)) {
                Ok(duration) => JobVariant::labeled(WorkerJob::UpdatePricesAll, slug).every(duration),
                Err(_) => JobVariant::labeled(WorkerJob::UpdatePricesAll, slug),
            };
            builder.job(
                prices_variant,
                provider_job(database, provider_instance, producer_prices.clone(), |u| async move { u.update_prices_all().await }),
            )
        }
    }
}

fn provider_job<F, Fut>(
    database: &Database,
    provider: Arc<dyn PriceAssetsProvider>,
    producer: StreamProducer,
    run: F,
) -> impl Fn(job_runner::JobContext) -> futures::future::BoxFuture<'static, Result<usize, Box<dyn std::error::Error + Send + Sync>>> + Clone + Send + Sync + 'static
where
    F: Fn(PricesUpdater) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<usize, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
{
    let database = database.clone();
    move |_| {
        let database = database.clone();
        let provider = provider.clone();
        let producer = producer.clone();
        let run = run.clone();
        Box::pin(async move { run(PricesUpdater::new(provider, database, producer)).await })
    }
}

#[derive(Clone, Copy)]
enum ChartsAction {
    Aggregate(ChartTimeframe),
    Cleanup(ChartTimeframe),
}

fn charts_job(
    database: &Database,
    cacher: &CacherClient,
    action: ChartsAction,
) -> impl Fn(job_runner::JobContext) -> futures::future::BoxFuture<'static, Result<usize, Box<dyn std::error::Error + Send + Sync>>> + Clone + Send + Sync + 'static {
    let updater = ChartsUpdater::new(PriceClient::new(database.clone(), cacher.clone()));
    move |_| {
        let updater = updater.clone();
        Box::pin(async move {
            match action {
                ChartsAction::Aggregate(tf) => updater.aggregate_charts(tf).await,
                ChartsAction::Cleanup(tf) => updater.cleanup_charts(tf).await,
            }
        })
    }
}

fn build_provider(provider: PriceProvider, settings: &Settings) -> Arc<dyn PriceAssetsProvider> {
    match provider {
        PriceProvider::Coingecko => Arc::new(CoinGeckoPricesProvider::new(&settings.coingecko.key.secret)),
        PriceProvider::Pyth => Arc::new(PythProvider::new(ReqwestClient::new(settings.prices.pyth.url.clone(), reqwest::Client::new()))),
        PriceProvider::Jupiter => Arc::new(JupiterProvider::new(ReqwestClient::new(settings.prices.jupiter.url.clone(), reqwest::Client::new()))),
    }
}

async fn stream_producer(settings: &Settings, name: &str) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    StreamProducer::new(&config, name, streamer::no_shutdown()).await
}
