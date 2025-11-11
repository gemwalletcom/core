use gem_tracing::info_with_fields;
use prices_dex::PriceFeedProvider;
use primitives::{AddressType, Asset, AssetTag, AssetType, Chain, FiatProviderName, LinkType, NFTType, Platform, PlatformStore, Subscription, TransactionType};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use storage::Database;
use storage::models::{FiatRate, UpdateDevice};
use streamer::{ExchangeName, QueueName, StreamProducer};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let postgres_url = settings.postgres.url.as_str();
    let database: Database = Database::new(postgres_url, settings.postgres.pool);
    database.client()?.migrations().run_migrations().unwrap();
    info_with_fields!("setup", step = "postgres migrations complete");

    let chains = Chain::all();

    info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

    info_with_fields!("setup", step = "add chains");
    let _ = database
        .client()?
        .assets()
        .add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    info_with_fields!("setup", step = "parser state");
    for chain in chains.clone() {
        let _ = database.client()?.parser_state().add_parser_state(chain.as_ref());
    }

    info_with_fields!("setup", step = "assets_types");

    let assets_types = AssetType::all();
    let _ = database.client()?.assets_types().add_assets_types(assets_types);

    info_with_fields!("setup", step = "assets");
    let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database.client()?.assets().add_assets(assets);

    info_with_fields!("setup", step = "fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProvider::from_primitive)
        .collect::<Vec<_>>();
    let _ = database.client()?.fiat().add_fiat_providers(providers);

    info_with_fields!("setup", step = "releases");

    let releases = PlatformStore::all()
        .into_iter()
        .map(|x| storage::models::Release {
            platform_store: x.as_ref().to_string(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
        })
        .collect::<Vec<_>>();

    let _ = database.client()?.releases().add_releases(releases);

    info_with_fields!("setup", step = "nft types");
    let types = NFTType::all().into_iter().map(storage::models::NftType::from_primitive).collect::<Vec<_>>();
    let _ = database.client()?.nft().add_nft_types(types);

    info_with_fields!("setup", step = "link types");
    let _ = database.client()?.link_types().add_link_types(LinkType::all());

    info_with_fields!("setup", step = "scan address types");
    let address_types = AddressType::all()
        .into_iter()
        .map(storage::models::ScanAddressType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database.client()?.scan_addresses().add_scan_address_types(address_types);

    info_with_fields!("setup", step = "transaction types");
    let address_types = TransactionType::all()
        .into_iter()
        .map(storage::models::TransactionType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database.client()?.transactions().add_transactions_types(address_types);

    info_with_fields!("setup", step = "assets tags");
    let assets_tags = AssetTag::all().into_iter().map(storage::models::Tag::from_primitive).collect::<Vec<_>>();
    let _ = database.client()?.tag().add_tags(assets_tags);

    info_with_fields!("setup", step = "prices dex providers");
    let providers = PriceFeedProvider::all()
        .into_iter()
        .enumerate()
        .map(|(index, p)| storage::models::PriceDexProvider::new(p.as_ref().to_string(), index as i32))
        .collect::<Vec<_>>();
    let _ = database.client()?.prices_dex().add_prices_dex_providers(providers);

    info_with_fields!(
        "setup",
        step = "search index",
        indexes = format!("{:?}", INDEX_CONFIGS.iter().map(|c| c.name).collect::<Vec<_>>())
    );

    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    search_index_client.setup(INDEX_CONFIGS, INDEX_PRIMARY_KEY).await.unwrap();

    info_with_fields!("setup", step = "queues");

    let queues = QueueName::all();
    let exchanges = ExchangeName::all();
    let stream_producer = StreamProducer::new(&settings.rabbitmq.url, "setup").await.unwrap();
    let _ = stream_producer.declare_queues(queues.clone()).await;
    let _ = stream_producer.declare_exchanges(exchanges.clone()).await;
    let _ = stream_producer
        .bind_exchange(
            ExchangeName::NewAddresses.clone(),
            vec![
                QueueName::FetchTokenAssociations,
                QueueName::FetchCoinAssociations,
                QueueName::FetchAddressTransactions,
                QueueName::FetchNftAssociations,
            ],
        )
        .await;

    info_with_fields!("setup", step = "complete");
    Ok(())
}

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let postgres_url = settings.postgres.url.as_str();
    let database: Database = Database::new(postgres_url, settings.postgres.pool);

    info_with_fields!("setup_dev", step = "add currency");

    let fiat_rate = FiatRate {
        id: "USD".to_string(),
        name: "US Dollar".to_string(),
        rate: 1.0,
    };

    info_with_fields!("setup_dev", step = "add rate", currency = "USD");
    let _ = database.client()?.set_fiat_rates(vec![fiat_rate.clone()]).expect("Failed to add currency");

    info_with_fields!("setup_dev", step = "add device");

    let device = UpdateDevice {
        device_id: "test".to_string(),
        platform: Platform::IOS.as_ref().to_string(),
        platform_store: Some(PlatformStore::AppStore.as_ref().to_string()),
        token: "test_token".to_string(),
        locale: "en".to_string(),
        currency: fiat_rate.id.clone(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: Some("iOS 18".to_string()),
        model: Some("iPhone 16".to_string()),
    };

    let _ = database.client()?.add_device(device).expect("Failed to add device");
    info_with_fields!("setup_dev", step = "device added", device_id = "test");

    info_with_fields!("setup_dev", step = "add subscription");

    let subscription = Subscription {
        wallet_index: 1,
        chain: Chain::Ethereum,
        address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".to_string(),
    };

    let result = database
        .client()?
        .subscriptions()
        .add_subscriptions(vec![subscription], "test")
        .expect("Failed to add subscription");
    info_with_fields!("setup_dev", step = "subscription added", count = result);

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}
