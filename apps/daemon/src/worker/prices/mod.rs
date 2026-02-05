mod charts_updater;
mod markets_updater;
mod observed_prices_updater;
pub mod price_updater;
mod prices_dex_updater;

use crate::model::WorkerService;
use crate::worker::context::WorkerContext;
use crate::worker::jobs::{JobVariant, WorkerJob};
use crate::worker::plan::JobPlanBuilder;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::CacherClient;
use charts_updater::ChartsUpdater;
use coingecko::CoinGeckoClient;
use job_runner::{JobHandle, ShutdownReceiver};
use markets_updater::MarketsUpdater;
use observed_prices_updater::ObservedPricesUpdater;
use price_updater::{PriceUpdater, UpdatePrices};
use pricer::{MarketsClient, PriceClient};
use prices_dex::PriceFeedProvider;
use prices_dex_updater::PricesDexUpdater;
use primitives::ConfigKey;
use settings::Settings;
use storage::{ConfigCacher, Database};
use streamer::{StreamProducer, StreamProducerConfig};

struct DexProviderConfig {
    provider_type: PriceFeedProvider,
    name: &'static str,
    url: String,
    timer: u64,
}

pub async fn jobs(ctx: WorkerContext, shutdown_rx: ShutdownReceiver) -> Result<Vec<JobHandle>, Box<dyn Error + Send + Sync>> {
    let runtime = ctx.runtime();
    let database = ctx.database();
    let settings = ctx.settings();
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let config = Arc::new(ConfigCacher::new(database.clone()));

    let dex_providers = vec![
        DexProviderConfig {
            provider_type: PriceFeedProvider::Pyth,
            name: "Pyth",
            url: settings.prices.pyth.url.clone(),
            timer: settings.prices.pyth.timer,
        },
        DexProviderConfig {
            provider_type: PriceFeedProvider::Jupiter,
            name: "Jupiter",
            url: settings.prices.jupiter.url.clone(),
            timer: settings.prices.jupiter.timer,
        },
    ];

    let builder = JobPlanBuilder::with_config(WorkerService::Prices, runtime.plan(shutdown_rx), config.as_ref())
        .job(WorkerJob::CleanupOutdatedAssets, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            let config = config.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                let config = config.clone();
                async move {
                    let price_outdated = config.get_duration(ConfigKey::PriceOutdated)?.as_secs();
                    price_updater_factory(&database, &cacher_client, &settings)
                        .await?
                        .clean_outdated_assets(price_outdated)
                        .await
                }
            }
        })
        .job(WorkerJob::UpdateFiatRates, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move { price_updater_factory(&database, &cacher_client, &settings).await?.update_fiat_rates().await }
            }
        })
        .job(WorkerJob::UpdatePricesTopMarketCap, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    price_updater_factory(&database, &cacher_client, &settings)
                        .await?
                        .update_prices_type(UpdatePrices::Top)
                        .await
                }
            }
        })
        .job(WorkerJob::UpdatePricesHighMarketCap, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    price_updater_factory(&database, &cacher_client, &settings)
                        .await?
                        .update_prices_type(UpdatePrices::High)
                        .await
                }
            }
        })
        .job(WorkerJob::UpdatePricesLowMarketCap, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    price_updater_factory(&database, &cacher_client, &settings)
                        .await?
                        .update_prices_type(UpdatePrices::Low)
                        .await
                }
            }
        })
        .job(WorkerJob::AggregateHourlyCharts, {
            let settings = settings.clone();
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    charts_updater_factory(&database, &cacher_client, &settings, coingecko_client)
                        .await?
                        .aggregate_hourly_charts()
                        .await
                }
            }
        })
        .job(WorkerJob::AggregateDailyCharts, {
            let settings = settings.clone();
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    charts_updater_factory(&database, &cacher_client, &settings, coingecko_client)
                        .await?
                        .aggregate_daily_charts()
                        .await
                }
            }
        })
        .job(WorkerJob::CleanupChartsData, {
            let settings = settings.clone();
            let coingecko_client = coingecko_client.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let coingecko_client = coingecko_client.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move {
                    charts_updater_factory(&database, &cacher_client, &settings, coingecko_client)
                        .await?
                        .cleanup_charts_data()
                        .await
                }
            }
        })
        .job(WorkerJob::UpdateMarkets, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                async move { markets_updater_factory(&database, &cacher_client, &settings).update_markets().await }
            }
        })
        .job(WorkerJob::UpdateObservedPrices, {
            let settings = settings.clone();
            let cacher_client = cacher_client.clone();
            let database = database.clone();
            let config = config.clone();
            move || {
                let settings = settings.clone();
                let cacher_client = cacher_client.clone();
                let database = database.clone();
                let config = config.clone();
                async move {
                    let max_observed_assets = config.get_usize(ConfigKey::PriceObservedMaxAssets)?;
                    let min_observers = config.get_usize(ConfigKey::PriceObservedMinObservers)?;
                    let price_client = PriceClient::new(database.clone(), cacher_client.clone());
                    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
                    let stream_producer = StreamProducer::new(&rabbitmq_config, "observed_prices_worker").await?;
                    let updater = ObservedPricesUpdater::new(cacher_client, price_client, stream_producer, max_observed_assets, min_observers);
                    updater.update().await
                }
            }
        });

    dex_providers
        .into_iter()
        .fold(builder, |builder, provider| {
            let slug = provider.name.to_lowercase();
            let builder = builder.job(JobVariant::labeled(WorkerJob::UpdateDexFeeds, slug.clone()).every(Duration::from_secs(3600)), {
                let url = provider.url.clone();
                let database = database.clone();
                let provider_type = provider.provider_type.clone();
                move || {
                    let url = url.clone();
                    let database = database.clone();
                    let provider_type = provider_type.clone();
                    async move { PricesDexUpdater::new(provider_type, &url, database).update_feeds().await }
                }
            });

            builder.job(JobVariant::labeled(WorkerJob::UpdateDexPrices, slug).every(Duration::from_secs(provider.timer)), {
                let url = provider.url.clone();
                let database = database.clone();
                let provider_type = provider.provider_type.clone();
                move || {
                    let url = url.clone();
                    let database = database.clone();
                    let provider_type = provider_type.clone();
                    async move { PricesDexUpdater::new(provider_type, &url, database).update_prices().await }
                }
            })
        })
        .finish()
}

async fn price_updater_factory(database: &Database, cacher: &CacherClient, settings: &Settings) -> Result<PriceUpdater, Box<dyn std::error::Error + Send + Sync>> {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let price_client = PriceClient::new(database.clone(), cacher.clone());
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "pricer_worker").await?;
    Ok(PriceUpdater::new(price_client, coingecko_client, stream_producer))
}

async fn charts_updater_factory(
    database: &Database,
    cacher: &CacherClient,
    settings: &Settings,
    coingecko_client: CoinGeckoClient,
) -> Result<ChartsUpdater, Box<dyn std::error::Error + Send + Sync>> {
    let price_client = PriceClient::new(database.clone(), cacher.clone());
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "charts_worker").await?;
    Ok(ChartsUpdater::new(price_client, coingecko_client, stream_producer))
}

fn markets_updater_factory(database: &Database, cacher: &CacherClient, settings: &Settings) -> MarketsUpdater {
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret.clone());
    let markets_client = MarketsClient::new(database.clone(), cacher.clone());
    MarketsUpdater::new(markets_client, coingecko_client)
}
