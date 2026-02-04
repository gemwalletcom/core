pub mod assets_addresses_consumer;
pub mod consumer_reporter;
pub mod fetch_address_transactions_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_prices_consumer;
pub mod fetch_token_addresses_consumer;
pub mod nft;
pub mod notifications;
pub mod rewards_consumer;
pub mod rewards_redemption_consumer;
pub mod store_charts_consumer;
pub mod store_prices_consumer;
pub mod store_transactions_consumer;
pub mod store_transactions_consumer_config;
pub mod support;

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

pub use assets_addresses_consumer::AssetsAddressesConsumer;
use cacher::CacherClient;
use gem_tracing::error_with_fields;
pub use fetch_assets_consumer::FetchAssetsConsumer;
use pricer::PriceClient;
use primitives::ConfigKey;
use settings::Settings;
use settings_chain::ChainProviders;
use storage::{ConfigCacher, Database};
pub use store_charts_consumer::StoreChartsConsumer;
pub use store_prices_consumer::StorePricesConsumer;
pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;
use streamer::{
    AssetsAddressPayload, ChainAddressPayload, ChartsPayload, ConsumerConfig, ConsumerStatusReporter, FetchAssetsPayload, FetchBlocksPayload, FetchPricesPayload,
    FiatWebhookPayload, InAppNotificationPayload, PricesPayload, QueueName, RewardsNotificationPayload, RewardsRedemptionPayload, ShutdownReceiver, StreamConnection,
    StreamProducer, StreamProducerConfig, StreamReader, StreamReaderConfig, SupportWebhookPayload, TransactionsPayload, run_consumer,
};

use crate::consumers::{
    fetch_address_transactions_consumer::FetchAddressTransactionsConsumer, fetch_blocks_consumer::FetchBlocksConsumer, fetch_coin_addresses_consumer::FetchCoinAddressesConsumer,
    fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer, fetch_prices_consumer::FetchPricesConsumer, fetch_token_addresses_consumer::FetchTokenAddressesConsumer,
};
use crate::pusher::Pusher;
use crate::worker::pricer::price_updater::PriceUpdater;
use coingecko::CoinGeckoClient;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use gem_rewards::{EvmClientProvider, TransferRedemptionService, WalletConfig};
use primitives::rewards::RedemptionStatus;
use primitives::{Chain, ChainType, EVMChain, NFTChain};
use settings::service_user_agent;
use settings_chain::ProviderFactory;

pub fn chain_providers(settings: &Settings, name: &str) -> ChainProviders {
    ChainProviders::from_settings(settings, &service_user_agent("consumer", Some(name)))
}

pub(crate) fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
        delay: consumer.delay,
    }
}

async fn reader_for_queue(settings: &Settings, queue: &QueueName) -> Result<(String, StreamReader), Box<dyn Error + Send + Sync>> {
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let reader = StreamReader::new(config).await?;
    Ok((name, reader))
}

fn producer_config(settings: &Settings) -> StreamProducerConfig {
    StreamProducerConfig::new(settings.rabbitmq.url.clone(), settings.rabbitmq.retry_delay, settings.rabbitmq.retry_max_delay)
}

async fn producer_for_queue(settings: &Settings, name: &str) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
    let config = producer_config(settings);
    StreamProducer::new(&config, name).await
}

#[derive(Clone)]
struct ChainConsumerRunner {
    settings: Settings,
    database: Database,
    connection: StreamConnection,
    cacher: CacherClient,
    config: ConsumerConfig,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
}

impl ChainConsumerRunner {
    async fn new(settings: Settings, queue: QueueName, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let database = Database::new(&settings.postgres.url, settings.postgres.pool);
        let connection = StreamConnection::new(&settings.rabbitmq.url, queue.to_string()).await?;
        let cacher = CacherClient::new(&settings.redis.url).await;
        let config = consumer_config(&settings.consumer);
        Ok(Self {
            settings,
            database,
            connection,
            cacher,
            config,
            shutdown_rx,
            reporter,
        })
    }

    async fn stream_reader(&self) -> Result<StreamReader, Box<dyn Error + Send + Sync>> {
        StreamReader::from_connection(&self.connection, self.settings.rabbitmq.prefetch).await
    }

    async fn stream_producer(&self) -> Result<StreamProducer, Box<dyn Error + Send + Sync>> {
        StreamProducer::from_connection(&self.connection).await
    }

    async fn run<F, Fut>(self, f: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Self, Chain) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        self.run_for_chains(Chain::all(), f).await
    }

    async fn run_for_chains<F, Fut>(self, chains: Vec<Chain>, f: F) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        F: Fn(Self, Chain) -> Fut + Clone + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
    {
        let tasks = chains.into_iter().map(|chain| {
            let runner = self.clone();
            let f = f.clone();
            async move { (chain, f(runner, chain).await) }
        });

        for (chain, result) in futures::future::join_all(tasks).await {
            if let Err(err) = result {
                error_with_fields!("consumer chain error", &*err, chain = chain.as_ref());
            }
        }
        Ok(())
    }
}

pub async fn run_consumer_fetch_assets(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FetchAssets;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, &name),
        database,
        cacher,
    };
    run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

pub async fn run_consumer_store_transactions(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::StoreTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::StoreTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let database = Database::new(&runner.settings.postgres.url, runner.settings.postgres.pool);
            let consumer = StoreTransactionsConsumer {
                database: database.clone(),
                config_cacher: ConfigCacher::new(database.clone()),
                stream_producer,
                pusher: Pusher::new(database.clone()),
                config: StoreTransactionsConsumerConfig {},
            };
            run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_fetch_address_transactions(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::FetchAddressTransactions, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchAddressTransactions;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchAddressTransactionsConsumer::new(runner.database, chain_providers(&runner.settings, &name), stream_producer, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchAddressTransactionsConsumer, usize>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_fetch_blocks(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::FetchBlocks, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchBlocks;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchBlocksConsumer::new(chain_providers(&runner.settings, &name), stream_producer);
            run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_store_assets_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StoreAssetsAssociations;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = AssetsAddressesConsumer::new(database);
    run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter)
        .await
}

pub async fn run_consumer_fetch_token_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::FetchTokenAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchTokenAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let stream_producer = runner.stream_producer().await?;
            let consumer = FetchTokenAddressesConsumer::new(chain_providers(&runner.settings, &name), runner.database, stream_producer, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchTokenAddressesConsumer, usize>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_fetch_coin_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    ChainConsumerRunner::new(settings, QueueName::FetchCoinAssociations, shutdown_rx, reporter)
        .await?
        .run(|runner, chain| async move {
            let queue = QueueName::FetchCoinAssociations;
            let name = format!("{}.{}", queue, chain.as_ref());
            let stream_reader = runner.stream_reader().await?;
            let consumer = FetchCoinAddressesConsumer::new(chain_providers(&runner.settings, &name), runner.database, runner.cacher);
            run_consumer::<ChainAddressPayload, FetchCoinAddressesConsumer, String>(
                &name,
                stream_reader,
                queue,
                Some(chain.as_ref()),
                consumer,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_fetch_nft_associations(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chains: Vec<Chain> = NFTChain::all().into_iter().map(Into::into).collect();
    ChainConsumerRunner::new(settings, QueueName::FetchNftAssociations, shutdown_rx, reporter)
        .await?
        .run_for_chains(chains, |runner, chain| async move {
            FetchNftAssetsAddressesConsumer::run(
                runner.settings,
                runner.database,
                chain,
                &runner.connection,
                runner.cacher,
                runner.config,
                runner.shutdown_rx,
                runner.reporter,
            )
            .await
        })
        .await
}

pub async fn run_consumer_support(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use support::support_webhook_consumer::SupportWebhookConsumer;
    let queue = QueueName::SupportWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}

pub async fn run_consumer_fiat(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::worker::fiat::fiat_webhook_consumer::FiatWebhookConsumer;
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FiatOrderWebhooks;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let consumer = FiatWebhookConsumer::new(database, settings.clone());
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<FiatWebhookPayload, FiatWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}

pub async fn run_consumer_store_prices(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StorePrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database.clone(), cacher_client);
    let config = ConfigCacher::new(database.clone());
    let ttl_seconds = config.get_duration(ConfigKey::PriceOutdated)?.as_secs() as i64;
    let consumer = StorePricesConsumer::new(database, price_client, ttl_seconds);
    run_consumer::<PricesPayload, StorePricesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

pub async fn run_consumer_store_charts(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::StoreCharts;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database, cacher_client);
    let consumer = StoreChartsConsumer::new(price_client);
    run_consumer::<ChartsPayload, StoreChartsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}

pub async fn run_consumer_rewards(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::RewardsEvents;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let consumer = rewards_consumer::RewardsConsumer::new(database, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<RewardsNotificationPayload, rewards_consumer::RewardsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
}

pub async fn run_consumer_in_app_notifications(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::NotificationsInApp;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let consumer = notifications::InAppNotificationsConsumer::new(database, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<InAppNotificationPayload, notifications::InAppNotificationsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter)
        .await
}

pub async fn run_rewards_redemption_consumer(
    settings: Settings,
    shutdown_rx: ShutdownReceiver,
    reporter: Arc<dyn ConsumerStatusReporter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let config = ConfigCacher::new(database.clone());
    let retry_config = rewards_redemption_consumer::RedemptionRetryConfig {
        max_retries: config.get_i64(ConfigKey::RedemptionRetryMaxRetries)? as u32,
        delay: config.get_duration(ConfigKey::RedemptionRetryDelay)?,
        errors: config.get_vec_string(ConfigKey::RedemptionRetryErrors)?,
    };
    let queue = QueueName::RewardsRedemptions;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let wallets = parse_rewards_wallets(&settings)?;
    let client_provider = create_evm_client_provider(settings.clone());
    let redemption_service = Arc::new(TransferRedemptionService::new(wallets, client_provider));
    let consumer = rewards_redemption_consumer::RewardsRedemptionConsumer::new(database, redemption_service, retry_config, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<RewardsRedemptionPayload, rewards_redemption_consumer::RewardsRedemptionConsumer<TransferRedemptionService>, RedemptionStatus>(
        &name,
        stream_reader,
        queue,
        None,
        consumer,
        consumer_config,
        shutdown_rx,
        reporter,
    )
    .await
}

fn parse_rewards_wallets(settings: &Settings) -> Result<HashMap<ChainType, WalletConfig>, Box<dyn Error + Send + Sync>> {
    let mut wallets = HashMap::new();

    for (chain_type_name, wallet_config) in &settings.rewards.wallets {
        let chain_type = ChainType::from_str(chain_type_name).map_err(|_| format!("Invalid chain type: {}", chain_type_name))?;
        wallets.insert(
            chain_type,
            WalletConfig {
                key: wallet_config.key.clone(),
                address: wallet_config.address.clone(),
            },
        );
    }

    Ok(wallets)
}

fn create_evm_client_provider(settings: Settings) -> EvmClientProvider {
    Arc::new(move |chain: EVMChain| {
        let chain_config = ProviderFactory::get_chain_config(chain.to_chain(), &settings);
        let reqwest_client = gem_client::builder().build().ok()?;
        let client = ReqwestClient::new(chain_config.url.clone(), reqwest_client);
        let rpc_client = JsonRpcClient::new(client);
        Some(EthereumClient::new(rpc_client, chain))
    })
}

pub async fn run_consumer_fetch_prices(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::FetchPrices;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let coingecko_client = CoinGeckoClient::new(&settings.coingecko.key.secret);
    let price_client = PriceClient::new(database, cacher_client);
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let price_updater = PriceUpdater::new(price_client, coingecko_client, stream_producer);
    let consumer = FetchPricesConsumer::new(price_updater);
    run_consumer::<FetchPricesPayload, FetchPricesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer), shutdown_rx, reporter).await
}
