pub mod assets_addresses_consumer;
pub mod fetch_address_transactions_consumer;
pub mod fetch_assets_consumer;
pub mod fetch_blocks_consumer;
pub mod fetch_coin_addresses_consumer;
pub mod fetch_nft_assets_addresses_consumer;
pub mod fetch_token_addresses_consumer;
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
pub use fetch_assets_consumer::FetchAssetsConsumer;
use pricer::PriceClient;
use primitives::ConfigKey;
use settings::Settings;
use settings_chain::ChainProviders;
use storage::Database;
pub use store_charts_consumer::StoreChartsConsumer;
pub use store_prices_consumer::StorePricesConsumer;
pub use store_transactions_consumer::StoreTransactionsConsumer;
pub use store_transactions_consumer_config::StoreTransactionsConsumerConfig;
use streamer::{
    AssetsAddressPayload, ChainAddressPayload, ChartsPayload, ConsumerConfig, FetchAssetsPayload, FetchBlocksPayload, FiatWebhookPayload, PricesPayload,
    QueueName, RewardsNotificationPayload, RewardsRedemptionPayload, StreamProducer, StreamReader, StreamReaderConfig, SupportWebhookPayload,
    TransactionsPayload, run_consumer,
};

use crate::consumers::{
    fetch_address_transactions_consumer::FetchAddressTransactionsConsumer, fetch_blocks_consumer::FetchBlocksConsumer,
    fetch_coin_addresses_consumer::FetchCoinAddressesConsumer, fetch_nft_assets_addresses_consumer::FetchNftAssetsAddressesConsumer,
    fetch_token_addresses_consumer::FetchTokenAddressesConsumer,
};
use crate::pusher::Pusher;
use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use gem_rewards::{EvmClientProvider, TransferRedemptionService, WalletConfig};
use primitives::{Chain, ChainType, EVMChain};
use settings::service_user_agent;
use settings_chain::ProviderFactory;

pub fn chain_providers(settings: &Settings, name: &str) -> ChainProviders {
    ChainProviders::from_settings(settings, &service_user_agent("consumer", Some(name)))
}

fn consumer_config(consumer: &settings::Consumer) -> ConsumerConfig {
    ConsumerConfig {
        timeout_on_error: consumer.error.timeout,
        skip_on_error: consumer.error.skip,
    }
}

pub async fn run_consumer_fetch_assets(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchAssets;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAssetsConsumer {
        providers: chain_providers(&settings, &name),
        database,
        cacher,
    };
    run_consumer::<FetchAssetsPayload, FetchAssetsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer)).await
}

pub async fn run_consumer_store_transactions(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreTransactions;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = StoreTransactionsConsumer {
        database: database.clone(),
        stream_producer,
        pusher: Pusher::new(database.clone()),
        config: StoreTransactionsConsumerConfig {},
    };
    run_consumer::<TransactionsPayload, StoreTransactionsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer))
        .await
}

async fn run_for_all_chains<F, Fut>(f: F) -> Result<(), Box<dyn Error + Send + Sync>>
where
    F: Fn(Chain) -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    let tasks: Vec<_> = Chain::all().into_iter().map(|chain| tokio::spawn(f(chain))).collect();
    for result in futures::future::join_all(tasks).await {
        result??;
    }
    Ok(())
}

pub async fn run_consumer_fetch_address_transactions(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_for_all_chains(|chain| {
        let settings = settings.clone();
        let database = database.clone();
        async move { run_consumer_fetch_address_transactions_chain(settings, database, chain).await }
    })
    .await
}

async fn run_consumer_fetch_address_transactions_chain(settings: Settings, database: Database, chain: Chain) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchAddressTransactions;
    let name = format!("{}.{}", queue, chain.as_ref());
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchAddressTransactionsConsumer::new(database, chain_providers(&settings, &name), stream_producer, cacher);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<ChainAddressPayload, FetchAddressTransactionsConsumer, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, consumer_config)
        .await
}

pub async fn run_consumer_fetch_blocks(settings: Settings) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_for_all_chains(|chain| {
        let settings = settings.clone();
        async move { run_consumer_fetch_blocks_chain(settings, chain).await }
    })
    .await
}

async fn run_consumer_fetch_blocks_chain(settings: Settings, chain: Chain) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchBlocks;
    let name = format!("{}.{}", queue, chain.as_ref());
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = FetchBlocksConsumer::new(chain_providers(&settings, &name), stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<FetchBlocksPayload, FetchBlocksConsumer, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, consumer_config).await
}

pub async fn run_consumer_store_assets_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreAssetsAssociations;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = AssetsAddressesConsumer::new(database);
    run_consumer::<AssetsAddressPayload, AssetsAddressesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer))
        .await
}

pub async fn run_consumer_fetch_token_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_for_all_chains(|chain| {
        let settings = settings.clone();
        let database = database.clone();
        async move { run_consumer_fetch_token_associations_chain(settings, database, chain).await }
    })
    .await
}

async fn run_consumer_fetch_token_associations_chain(settings: Settings, database: Database, chain: Chain) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchTokenAssociations;
    let name = format!("{}.{}", queue, chain.as_ref());
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchTokenAddressesConsumer::new(chain_providers(&settings, &name), database, stream_producer, cacher);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<ChainAddressPayload, FetchTokenAddressesConsumer, usize>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, consumer_config).await
}

pub async fn run_consumer_fetch_coin_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_for_all_chains(|chain| {
        let settings = settings.clone();
        let database = database.clone();
        async move { run_consumer_fetch_coin_associations_chain(settings, database, chain).await }
    })
    .await
}

async fn run_consumer_fetch_coin_associations_chain(settings: Settings, database: Database, chain: Chain) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::FetchCoinAssociations;
    let name = format!("{}.{}", queue, chain.as_ref());
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher = CacherClient::new(&settings.redis.url).await;
    let consumer = FetchCoinAddressesConsumer::new(chain_providers(&settings, &name), database, cacher);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<ChainAddressPayload, FetchCoinAddressesConsumer, String>(&name, stream_reader, queue, Some(chain.as_ref()), consumer, consumer_config).await
}

pub async fn run_consumer_fetch_nft_associations(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = consumer_config(&settings.consumer);
    run_for_all_chains(|chain| {
        let settings = settings.clone();
        let database = database.clone();
        let config = config.clone();
        async move { FetchNftAssetsAddressesConsumer::run(settings, database, chain, config).await }
    })
    .await
}

pub async fn run_consumer_support(settings: Settings, _database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    use support::support_webhook_consumer::SupportWebhookConsumer;
    let queue = QueueName::SupportWebhooks;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = SupportWebhookConsumer::new(&settings).await?;
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<SupportWebhookPayload, SupportWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config).await
}

pub async fn run_consumer_fiat(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    use crate::worker::fiat::fiat_webhook_consumer::FiatWebhookConsumer;
    let queue = QueueName::FiatOrderWebhooks;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let consumer = FiatWebhookConsumer::new(database, settings.clone());
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<FiatWebhookPayload, FiatWebhookConsumer, bool>(&name, stream_reader, queue, None, consumer, consumer_config).await
}

pub async fn run_consumer_store_prices(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StorePrices;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database.clone(), cacher_client);
    let ttl_seconds = database.client()?.config().get_config_duration(ConfigKey::PricerOutdated)?.as_secs() as i64;
    let consumer = StorePricesConsumer::new(database, price_client, ttl_seconds);
    run_consumer::<PricesPayload, StorePricesConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer)).await
}

pub async fn run_consumer_store_charts(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::StoreCharts;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let cacher_client = CacherClient::new(&settings.redis.url).await;
    let price_client = PriceClient::new(database, cacher_client);
    let consumer = StoreChartsConsumer::new(price_client);
    run_consumer::<ChartsPayload, StoreChartsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config(&settings.consumer)).await
}

pub async fn run_consumer_rewards(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::RewardsEvents;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, &name).await?;
    let consumer = rewards_consumer::RewardsConsumer::new(database, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<RewardsNotificationPayload, rewards_consumer::RewardsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config).await
}

pub async fn run_rewards_redemption_consumer(settings: Settings, database: Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let queue = QueueName::RewardsRedemptions;
    let name = queue.to_string();
    let config = StreamReaderConfig::new(settings.rabbitmq.url.clone(), name.clone(), settings.rabbitmq.prefetch);
    let stream_reader = StreamReader::new(config).await?;
    let wallets = parse_rewards_wallets(&settings)?;
    let client_provider = create_evm_client_provider(settings.clone());
    let redemption_service = Arc::new(TransferRedemptionService::new(wallets, client_provider));
    let consumer = rewards_redemption_consumer::RewardsRedemptionConsumer::new(database, redemption_service);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<RewardsRedemptionPayload, rewards_redemption_consumer::RewardsRedemptionConsumer<TransferRedemptionService>, bool>(
        &name,
        stream_reader,
        queue,
        None,
        consumer,
        consumer_config,
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
