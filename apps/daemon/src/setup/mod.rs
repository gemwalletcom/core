use gem_tracing::info_with_fields;
use prices_dex::PriceFeedProvider;
use primitives::{Asset, AssetTag, Chain, ConfigKey, FiatProviderName, PlatformStore as PrimitivePlatformStore, Subscription};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use storage::Database;
use storage::models::{ConfigRow, FiatRateRow, UpdateDeviceRow};
use storage::sql_types::{Platform, PlatformStore};
use storage::{
    AssetsRepository, ChainsRepository, ConfigRepository, DevicesRepository, MigrationsRepository, PricesDexRepository, ReleasesRepository, SubscriptionsRepository, TagRepository,
};
use streamer::{ExchangeKind, ExchangeName, QueueName, StreamProducer};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let postgres_url = settings.postgres.url.as_str();
    let database: Database = Database::new(postgres_url, settings.postgres.pool);
    database.migrations()?.run_migrations().unwrap();
    info_with_fields!("setup", step = "postgres migrations complete");

    let chains = Chain::all();

    info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

    info_with_fields!("setup", step = "add chains");
    let _ = database.chains()?.add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    info_with_fields!("setup", step = "parser state");
    for chain in chains.clone() {
        let _ = database.parser_state()?.add_parser_state(chain.as_ref());
    }

    info_with_fields!("setup", step = "assets");
    let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database.assets()?.add_assets(assets);

    info_with_fields!("setup", step = "fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProviderRow::from_primitive)
        .collect::<Vec<_>>();
    let _ = database.fiat()?.add_fiat_providers(providers);

    info_with_fields!("setup", step = "releases");

    let releases = PrimitivePlatformStore::all()
        .into_iter()
        .map(|x| storage::models::ReleaseRow {
            platform_store: x.into(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
        })
        .collect::<Vec<_>>();

    let _ = database.releases()?.add_releases(releases);

    info_with_fields!("setup", step = "assets tags");
    let assets_tags = AssetTag::all().into_iter().map(storage::models::TagRow::from_primitive).collect::<Vec<_>>();
    let _ = database.tag()?.add_tags(assets_tags);

    info_with_fields!("setup", step = "prices dex providers");
    let providers = PriceFeedProvider::all()
        .into_iter()
        .enumerate()
        .map(|(index, p)| storage::models::PriceDexProviderRow::new(p.as_ref().to_string(), index as i32))
        .collect::<Vec<_>>();
    let _ = database.prices_dex()?.add_prices_dex_providers(providers);

    info_with_fields!("setup", step = "config");
    let configs = ConfigKey::all().into_iter().map(ConfigRow::from_primitive).collect::<Vec<_>>();
    let _ = database.client()?.add_config(configs);

    info_with_fields!(
        "setup",
        step = "search index",
        indexes = format!("{:?}", INDEX_CONFIGS.iter().map(|c| c.name).collect::<Vec<_>>())
    );

    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    search_index_client.setup(INDEX_CONFIGS, INDEX_PRIMARY_KEY).await.unwrap();

    info_with_fields!("setup", step = "queues");

    let chain_queues = QueueName::chain_queues();
    let non_chain_queues: Vec<_> = QueueName::all().into_iter().filter(|q| !chain_queues.contains(q)).collect();
    let exchanges = ExchangeName::all();
    let chains = Chain::all();

    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "setup").await.unwrap();
    let _ = stream_producer.declare_queues(non_chain_queues).await;
    let _ = stream_producer.declare_exchanges(exchanges.clone()).await;
    info_with_fields!(
        "setup",
        step = "queue exchanges for chain-based consumers",
        queues = format!("{:?}", chain_queues.iter().map(|q| q.to_string()).collect::<Vec<_>>()),
        chains = format!("{:?}", chains)
    );
    for queue in &chain_queues {
        let exchange_name = format!("{}_exchange", queue);
        let _ = stream_producer.declare_exchange(&exchange_name, ExchangeKind::Topic).await;
        for chain in &chains {
            let _ = stream_producer.bind_queue_routing_key(queue.clone(), chain.as_ref()).await;
        }
    }

    for exchange in &exchanges {
        let exchange_queues = exchange.queues();
        if exchange_queues.is_empty() {
            continue;
        }
        info_with_fields!(
            "setup",
            step = "exchange bindings",
            exchange = exchange.to_string(),
            queues = format!("{:?}", exchange_queues.iter().map(|q| q.to_string()).collect::<Vec<_>>())
        );
        for queue in &exchange_queues {
            for chain in &chains {
                let queue_name = format!("{}.{}", queue, chain.as_ref());
                let _ = stream_producer.bind_queue(&queue_name, &exchange.to_string(), chain.as_ref()).await;
            }
        }
    }

    info_with_fields!("setup", step = "complete");
    Ok(())
}

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let postgres_url = settings.postgres.url.as_str();
    let database: Database = Database::new(postgres_url, settings.postgres.pool);

    info_with_fields!("setup_dev", step = "add currency");

    let fiat_rate = FiatRateRow {
        id: "USD".to_string(),
        name: "US Dollar".to_string(),
        rate: 1.0,
    };

    info_with_fields!("setup_dev", step = "add rate", currency = "USD");
    let _ = database.fiat()?.set_fiat_rates(vec![fiat_rate.clone()]).expect("Failed to add currency");

    info_with_fields!("setup_dev", step = "add devices");

    let ios_device = UpdateDeviceRow {
        device_id: "test".to_string(),
        platform: Platform::IOS,
        platform_store: PlatformStore::AppStore,
        token: "test_token".to_string(),
        locale: "en".to_string(),
        currency: fiat_rate.id.clone(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "iOS 18".to_string(),
        model: "iPhone 16".to_string(),
    };

    let android_device = UpdateDeviceRow {
        device_id: "test-android".to_string(),
        platform: Platform::Android,
        platform_store: PlatformStore::GooglePlay,
        token: "test_token_android".to_string(),
        locale: "en".to_string(),
        currency: fiat_rate.id.clone(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "Android 15".to_string(),
        model: "Pixel 9".to_string(),
    };

    let _ = database.devices()?.add_device(ios_device).expect("Failed to add iOS device");
    info_with_fields!("setup_dev", step = "device added", device_id = "test");

    let _ = database.devices()?.add_device(android_device).expect("Failed to add Android device");
    info_with_fields!("setup_dev", step = "device added", device_id = "test-android");

    info_with_fields!("setup_dev", step = "add subscription");

    let subscription = Subscription {
        wallet_index: 1,
        chain: Chain::Ethereum,
        address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".to_string(),
    };

    let result = database.subscriptions()?.add_subscriptions(vec![subscription], "test").expect("Failed to add subscription");
    info_with_fields!("setup_dev", step = "subscription added", count = result);

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}
