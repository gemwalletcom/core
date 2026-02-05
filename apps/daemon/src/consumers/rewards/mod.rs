pub mod rewards_consumer;
pub mod rewards_redemption_consumer;

use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;

use gem_client::ReqwestClient;
use gem_evm::rpc::EthereumClient;
use gem_jsonrpc::JsonRpcClient;
use gem_rewards::{EvmClientProvider, TransferRedemptionService, WalletConfig};
use primitives::rewards::RedemptionStatus;
use primitives::{ChainType, ConfigKey, EVMChain};
use settings::Settings;
use settings_chain::ProviderFactory;
use storage::{ConfigCacher, Database};
use streamer::{ConsumerStatusReporter, QueueName, RewardsNotificationPayload, RewardsRedemptionPayload, ShutdownReceiver, run_consumer};

use crate::consumers::{consumer_config, producer_for_queue, reader_for_queue};

pub async fn run_consumer_rewards(settings: Settings, shutdown_rx: ShutdownReceiver, reporter: Arc<dyn ConsumerStatusReporter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let database = Database::new(&settings.postgres.url, settings.postgres.pool);
    let queue = QueueName::RewardsEvents;
    let (name, stream_reader) = reader_for_queue(&settings, &queue).await?;
    let stream_producer = producer_for_queue(&settings, &name).await?;
    let consumer = rewards_consumer::RewardsConsumer::new(database, stream_producer);
    let consumer_config = consumer_config(&settings.consumer);
    run_consumer::<RewardsNotificationPayload, rewards_consumer::RewardsConsumer, usize>(&name, stream_reader, queue, None, consumer, consumer_config, shutdown_rx, reporter).await
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
