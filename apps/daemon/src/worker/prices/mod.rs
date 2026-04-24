mod charts_updater;
mod markets_updater;
mod observed_prices_updater;
mod prices_cleanup_updater;
mod prices_metrics_updater;
pub mod prices_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::{JobVariant, WorkerJob};
use crate::worker::plan::JobPlanBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;

use cacher::CacherClient;
use charts_updater::{ChartsHistoryConfig, ChartsHistoryUpdater, ChartsUpdater};
use coingecko::CoinGeckoClient;
use gem_client::ReqwestClient;
use job_runner::{JobHandle, ShutdownReceiver};
use markets_updater::MarketsUpdater;
use observed_prices_updater::{ObservedPricesConfig, ObservedPricesUpdater};
use pricer::{MarketsClient, PriceClient};
use prices::{CoinGeckoPricesProvider, DefiLlamaProvider, JupiterProvider, PriceAssetsProvider, PriceProvider, PriceProviderConfig, PythProvider};
use prices_cleanup_updater::PricesCleanupUpdater;
use prices_metrics_updater::PricesMetricsUpdater;
use prices_updater::PricesUpdater;
use primitives::{ChartTimeframe, ConfigKey, ConfigParamKey};
use settings::Settings;
use storage::repositories::prices_providers_repository::PricesProvidersRepository;
use storage::{ConfigCacher, Database};
use streamer::{StreamProducer, StreamProducerConfig};

pub type AssetsProviders = Arc<HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>>;

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
    let assets_providers: AssetsProviders = Arc::new(build_providers(&enabled_providers, &settings, config.as_ref())?);

    let builder = JobPlanBuilder::with_config(WorkerService::Prices, runtime.plan(shutdown_rx), config.as_ref());
    let builder = if only.is_none() {
        add_platform_jobs(builder, &database, &cacher_client, &config, &assets_providers, producer_prices.clone())?
    } else {
        builder
    };
    enabled_providers
        .into_iter()
        .filter(|p| only.is_none_or(|o| o == *p))
        .try_fold(builder, |builder, provider| {
            add_provider_jobs(
                builder,
                &database,
                &cacher_client,
                &settings,
                &config,
                provider,
                assets_providers[&provider].clone(),
                &producer_assets,
                &producer_prices,
            )
        })?
        .finish()
}

fn add_platform_jobs<'a>(
    builder: JobPlanBuilder<'a>,
    database: &Database,
    cacher_client: &CacherClient,
    config: &Arc<ConfigCacher>,
    providers: &AssetsProviders,
    producer: StreamProducer,
) -> Result<JobPlanBuilder<'a>, Box<dyn Error + Send + Sync>> {
    Ok(builder
        .job(
            WorkerJob::AggregateHourlyCharts,
            charts_job(database, cacher_client, config, ChartsAction::Aggregate(ChartTimeframe::Hourly)),
        )
        .job(
            WorkerJob::AggregateDailyCharts,
            charts_job(database, cacher_client, config, ChartsAction::Aggregate(ChartTimeframe::Daily)),
        )
        .job(
            WorkerJob::CleanupChartsRaw,
            charts_job(database, cacher_client, config, ChartsAction::Delete(ChartTimeframe::Raw)),
        )
        .job(
            WorkerJob::CleanupChartsHourly,
            charts_job(database, cacher_client, config, ChartsAction::Delete(ChartTimeframe::Hourly)),
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
                    let observed_config = ObservedPricesConfig {
                        max_assets: config.get_usize(ConfigKey::PriceObservedMaxAssets)?,
                        min_observers: config.get_usize(ConfigKey::PriceObservedMinObservers)?,
                        primary_price_max_age: config.get_duration(ConfigKey::PricePrimaryMaxAge)?,
                    };
                    ObservedPricesUpdater::new(cacher_client, database, providers, producer, observed_config).update().await
                }
            }
        }))
}

fn add_provider_jobs<'a>(
    builder: JobPlanBuilder<'a>,
    database: &Database,
    cacher_client: &CacherClient,
    settings: &Settings,
    config: &Arc<ConfigCacher>,
    kind: PriceProvider,
    provider: Arc<dyn PriceAssetsProvider>,
    producer_assets: &StreamProducer,
    producer_prices: &StreamProducer,
) -> Result<JobPlanBuilder<'a>, Box<dyn Error + Send + Sync>> {
    let mut builder = builder;
    builder = add_updater_job(
        builder,
        database,
        &provider,
        producer_assets,
        config,
        kind,
        WorkerJob::UpdatePricesAssets,
        ConfigParamKey::PriceProviderAssetsDuration(kind),
        |u| async move { u.update_assets().await },
    )?;
    builder = add_updater_job(
        builder,
        database,
        &provider,
        producer_assets,
        config,
        kind,
        WorkerJob::UpdatePricesAssetsNew,
        ConfigParamKey::PriceProviderAssetsNewDuration(kind),
        |u| async move { u.update_assets_new().await },
    )?;
    builder = add_updater_job(
        builder,
        database,
        &provider,
        producer_assets,
        config,
        kind,
        WorkerJob::UpdatePricesAssetsMetadata,
        ConfigParamKey::PriceProviderAssetsMetadataDuration(kind),
        |u| async move { u.update_assets_metadata().await },
    )?;

    let cleanup_variant = JobVariant::labeled(WorkerJob::CleanupOutdatedAssets, kind).with_param_duration(config, &ConfigParamKey::PriceProviderCleanOutdatedDuration(kind))?;
    builder = builder.job(cleanup_variant, {
        let database = database.clone();
        let cacher_client = cacher_client.clone();
        let config = config.clone();
        move |_| {
            let updater = PricesCleanupUpdater::new(database.clone(), cacher_client.clone(), config.clone(), kind);
            async move { updater.update().await }
        }
    });

    let metrics_variant = JobVariant::labeled(WorkerJob::UpdatePricesMetrics, kind).with_param_duration(config, &ConfigParamKey::PriceProviderMetricsDuration(kind))?;
    builder = builder.job(metrics_variant, {
        let database = database.clone();
        move |_| {
            let updater = PricesMetricsUpdater::new(database.clone(), kind);
            async move { updater.update().await }
        }
    });

    builder = builder.job(
        JobVariant::labeled(WorkerJob::UpdateChartsHistory, kind),
        charts_history_job(
            database,
            cacher_client,
            provider.clone(),
            ChartsHistoryConfig {
                hourly_duration: config.get_param_duration(&ConfigParamKey::PriceProviderChartsHourlyDuration(kind))?,
            },
        ),
    );

    builder = match kind {
        PriceProvider::Coingecko => builder
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesTop, kind),
                provider_job(database, provider.clone(), producer_prices.clone(), |u| async move { u.update_prices_window(0, 500).await }),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesHigh, kind),
                provider_job(
                    database,
                    provider.clone(),
                    producer_prices.clone(),
                    |u| async move { u.update_prices_window(500, 2500).await },
                ),
            )
            .job(
                JobVariant::labeled(WorkerJob::UpdatePricesLow, kind),
                provider_job(database, provider.clone(), producer_prices.clone(), |u| async move {
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
        PriceProvider::Pyth | PriceProvider::Jupiter | PriceProvider::DefiLlama => add_updater_job(
            builder,
            database,
            &provider,
            producer_prices,
            config,
            kind,
            WorkerJob::UpdatePrices,
            ConfigParamKey::PriceProviderPricesDuration(kind),
            |u| async move { u.update_prices_all().await },
        )?,
    };
    Ok(builder)
}

#[allow(clippy::too_many_arguments)]
fn add_updater_job<'a, F, Fut>(
    builder: JobPlanBuilder<'a>,
    database: &Database,
    provider: &Arc<dyn PriceAssetsProvider>,
    producer: &StreamProducer,
    config: &ConfigCacher,
    kind: PriceProvider,
    job: WorkerJob,
    interval: ConfigParamKey,
    run: F,
) -> Result<JobPlanBuilder<'a>, Box<dyn Error + Send + Sync>>
where
    F: Fn(PricesUpdater) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<usize, Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    let variant = JobVariant::labeled(job, kind).with_param_duration(config, &interval)?;
    Ok(builder.job(variant, provider_job(database, provider.clone(), producer.clone(), run)))
}

fn build_providers(
    enabled_providers: &[PriceProvider],
    settings: &Settings,
    config: &ConfigCacher,
) -> Result<HashMap<PriceProvider, Arc<dyn PriceAssetsProvider>>, Box<dyn Error + Send + Sync>> {
    enabled_providers
        .iter()
        .copied()
        .map(|provider| build_provider(provider, settings, config).map(|provider_instance| (provider, provider_instance)))
        .collect()
}

fn build_provider(provider: PriceProvider, settings: &Settings, config: &ConfigCacher) -> Result<Arc<dyn PriceAssetsProvider>, Box<dyn Error + Send + Sync>> {
    let provider_config = PriceProviderConfig {
        min_score: config.get_param_f64(&ConfigParamKey::PriceProviderAssetsMinScore(provider))?,
    };
    Ok(match provider {
        PriceProvider::Coingecko => Arc::new(CoinGeckoPricesProvider::new(&settings.coingecko.key.secret, provider_config)),
        PriceProvider::Pyth => Arc::new(PythProvider::new(
            ReqwestClient::new(settings.prices.pyth.url.clone(), reqwest::Client::new()),
            provider_config,
        )),
        PriceProvider::Jupiter => Arc::new(JupiterProvider::new(
            ReqwestClient::new(settings.prices.jupiter.url.clone(), reqwest::Client::new()),
            provider_config,
        )),
        PriceProvider::DefiLlama => Arc::new(DefiLlamaProvider::new(ReqwestClient::new(
            settings.prices.defillama.url.clone(),
            reqwest::Client::new(),
        ))),
    })
}

fn charts_history_job(
    database: &Database,
    cacher: &CacherClient,
    provider: Arc<dyn PriceAssetsProvider>,
    config: ChartsHistoryConfig,
) -> impl Fn(job_runner::JobContext) -> futures::future::BoxFuture<'static, Result<usize, Box<dyn std::error::Error + Send + Sync>>> + Clone + Send + Sync + 'static {
    let database = database.clone();
    let cacher = cacher.clone();
    move |_| {
        let provider = provider.clone();
        let database = database.clone();
        let cacher = cacher.clone();
        Box::pin(async move { ChartsHistoryUpdater::new(provider, database, cacher, config).update().await })
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
    Delete(ChartTimeframe),
}

fn charts_job(
    database: &Database,
    cacher: &CacherClient,
    config: &Arc<ConfigCacher>,
    action: ChartsAction,
) -> impl Fn(job_runner::JobContext) -> futures::future::BoxFuture<'static, Result<usize, Box<dyn std::error::Error + Send + Sync>>> + Clone + Send + Sync + 'static {
    let updater = ChartsUpdater::new(PriceClient::new(database.clone(), cacher.clone()));
    let config = config.clone();
    move |_| {
        let updater = updater.clone();
        let config = config.clone();
        Box::pin(async move {
            match action {
                ChartsAction::Aggregate(tf) => updater.aggregate_charts(tf).await,
                ChartsAction::Delete(tf) => {
                    let retention = config.get_duration(charts_retention_key(tf))?;
                    let before = (chrono::Utc::now() - chrono::Duration::from_std(retention)?).naive_utc();
                    updater.delete_charts(tf, before).await
                }
            }
        })
    }
}

fn charts_retention_key(timeframe: ChartTimeframe) -> ConfigKey {
    match timeframe {
        ChartTimeframe::Raw => ConfigKey::PriceChartsRetentionRaw,
        ChartTimeframe::Hourly => ConfigKey::PriceChartsRetentionHourly,
        ChartTimeframe::Daily => ConfigKey::PriceChartsRetentionDaily,
    }
}

async fn stream_producer(settings: &Settings, name: &str) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    StreamProducer::new(&config, name, streamer::no_shutdown()).await
}
