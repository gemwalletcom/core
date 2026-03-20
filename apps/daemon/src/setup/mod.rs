use gem_tracing::info_with_fields;
use prices_dex::PriceFeedProvider;
use primitives::{
    Asset, AssetId, AssetTag, Chain, ConfigKey, FiatProviderName, FiatQuoteType, FiatTransaction, FiatTransactionStatus, NFTChain, NotificationType, ParamConfigKey,
    PlatformStore as PrimitivePlatformStore, PriceAlert, PriceAlertDirection,
};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use storage::Database;
use storage::models::{ChartRow, ConfigRow, FiatAssetRow, FiatProviderCountryRow, FiatRateRow, PriceAssetRow, PriceRow, UpdateDeviceRow};
use storage::sql_types::{Platform, PlatformStore};
use storage::{
    AssetsRepository, ChainsRepository, ChartsRepository, ConfigRepository, DevicesRepository, FiatRepository, MigrationsRepository, NewNotificationRow, NewWalletRow,
    NotificationsRepository, PriceAlertsRepository, PricesDexRepository, PricesRepository, ReleasesRepository, RewardsRepository, TagRepository, WalletSource, WalletType,
    WalletsRepository,
};
use streamer::{ExchangeKind, ExchangeName, QueueName, StreamProducer, StreamProducerConfig};

pub async fn run_setup(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "init");

    let database: Database = Database::new(&settings.postgres.url, settings.postgres.pool);

    setup_database(&database)?;
    setup_search_index(&settings).await?;
    setup_queues(&settings).await?;

    info_with_fields!("setup", step = "complete");
    Ok(())
}

fn setup_database(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    database.migrations()?.run_migrations().unwrap();
    info_with_fields!("setup", step = "postgres migrations complete");

    let chains = Chain::all();
    info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

    info_with_fields!("setup", step = "add chains");
    let _ = database.chains()?.add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    info_with_fields!("setup", step = "parser state");
    for chain in chains.clone() {
        let _ = database.parser_state()?.add_parser_state(chain, chain.block_time() as i32);
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
            update_enabled: true,
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
    let configs: Vec<ConfigRow> = ConfigKey::all().into_iter().map(ConfigRow::from_primitive).collect();
    let _ = database.client()?.add_config(configs);

    info_with_fields!("setup", step = "param config");
    let param_configs: Vec<ConfigRow> = ParamConfigKey::all().into_iter().map(ConfigRow::from_param).collect();
    let _ = database.client()?.add_config(param_configs);

    Ok(())
}

async fn setup_search_index(settings: &Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!(
        "setup",
        step = "search index",
        indexes = format!("{:?}", INDEX_CONFIGS.iter().map(|c| c.name).collect::<Vec<_>>())
    );

    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());
    search_index_client.setup(INDEX_CONFIGS, INDEX_PRIMARY_KEY).await.unwrap();

    Ok(())
}

async fn setup_queues(settings: &Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup", step = "queues");

    let chain_queues = QueueName::chain_queues();
    let non_chain_queues: Vec<_> = QueueName::all().into_iter().filter(|q| !chain_queues.contains(q)).collect();
    let exchanges = ExchangeName::all();
    let chains = Chain::all();

    let retry = streamer::Retry::new(settings.rabbitmq.retry.delay, settings.rabbitmq.retry.timeout);
    let rabbitmq_config = StreamProducerConfig::new(settings.rabbitmq.url.clone(), retry);
    let stream_producer = StreamProducer::new(&rabbitmq_config, "setup", streamer::no_shutdown()).await.unwrap();
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
        for chain in queue_supported_chains(queue, &chains) {
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
            for chain in queue_supported_chains(queue, &chains) {
                let queue_name = format!("{}.{}", queue, chain.as_ref());
                let _ = stream_producer.bind_queue(&queue_name, &exchange.to_string(), chain.as_ref()).await;
            }
        }
    }

    Ok(())
}

fn queue_supported_chains(queue: &QueueName, all_chains: &[Chain]) -> Vec<Chain> {
    match queue {
        QueueName::FetchNftAssociations => NFTChain::all().into_iter().map(Into::into).collect(),
        _ => all_chains.to_vec(),
    }
}

pub async fn run_setup_dev(settings: Settings) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "init");

    let database: Database = Database::new(&settings.postgres.url, settings.postgres.pool);

    setup_dev_currency(&database)?;
    setup_dev_devices(&database)?;
    setup_dev_assets(&database)?;

    info_with_fields!("setup_dev", step = "complete");
    Ok(())
}

fn setup_dev_currency(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add currency");

    let fiat_rate = FiatRateRow {
        id: "USD".to_string(),
        name: "US Dollar".to_string(),
        rate: 1.0,
    };

    info_with_fields!("setup_dev", step = "add rate", currency = "USD");
    database.fiat()?.set_fiat_rates(vec![fiat_rate])?;

    Ok(())
}

fn setup_dev_devices(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add devices");

    let ios_device_id = "0".repeat(64);
    let android_device_id = "1".repeat(64);

    let ios_device = UpdateDeviceRow {
        device_id: ios_device_id.clone(),
        platform: Platform::IOS,
        platform_store: PlatformStore::AppStore,
        token: "test_token".to_string(),
        locale: "en".to_string(),
        currency: "USD".to_string(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "iOS 18".to_string(),
        model: "iPhone 16".to_string(),
    };

    let android_device = UpdateDeviceRow {
        device_id: android_device_id.clone(),
        platform: Platform::Android,
        platform_store: PlatformStore::GooglePlay,
        token: "test_token_android".to_string(),
        locale: "en".to_string(),
        currency: "USD".to_string(),
        is_push_enabled: true,
        is_price_alerts_enabled: true,
        version: "1.0.0".to_string(),
        subscriptions_version: 1,
        os: "Android 15".to_string(),
        model: "Pixel 9".to_string(),
    };

    database.devices()?.add_device(ios_device)?;
    info_with_fields!("setup_dev", step = "device added", device_id = ios_device_id.as_str());

    database.devices()?.add_device(android_device)?;
    info_with_fields!("setup_dev", step = "device added", device_id = android_device_id.as_str());

    let ios_device_row_id = database.devices()?.get_device_row_id(&ios_device_id)?;
    let android_device_row_id = database.devices()?.get_device_row_id(&android_device_id)?;

    let wallet_address = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

    info_with_fields!("setup_dev", step = "add wallet");
    let wallet_identifier = format!("multicoin_{}", wallet_address);
    let new_wallet = NewWalletRow {
        identifier: wallet_identifier,
        wallet_type: WalletType::Multicoin,
        source: WalletSource::Create,
    };
    let wallet = database.wallets()?.get_or_create_wallet(new_wallet)?;
    info_with_fields!("setup_dev", step = "wallet added", wallet_id = wallet.id);

    info_with_fields!("setup_dev", step = "add wallet subscriptions");
    let solana_address = "8wytzyCBXco7yqgrLDiecpEt452MSuNWRe7xsLgAAX1H";
    let subscriptions = vec![
        (wallet.id, Chain::Ethereum, wallet_address.to_string()),
        (wallet.id, Chain::HyperCore, wallet_address.to_string()),
        (wallet.id, Chain::Solana, solana_address.to_string()),
    ];

    let result = WalletsRepository::add_subscriptions(&mut database.wallets()?, ios_device_row_id, subscriptions.clone())?;
    info_with_fields!("setup_dev", step = "ios wallet subscription added", count = result);

    let result = WalletsRepository::add_subscriptions(&mut database.wallets()?, android_device_row_id, subscriptions)?;
    info_with_fields!("setup_dev", step = "android wallet subscription added", count = result);

    setup_dev_fiat_transactions(database, wallet_address, solana_address)?;

    info_with_fields!("setup_dev", step = "add rewards");
    let devices = database.wallets()?.get_devices_by_wallet_id(wallet.id)?;
    if let Some(device) = devices.first() {
        let result = database.rewards()?.create_reward(wallet.id, "gemcoder", device.id);
        match result {
            Ok((rewards, _)) => info_with_fields!("setup_dev", step = "rewards added", code = rewards.code.unwrap_or_default(), points = rewards.points),
            Err(e) => info_with_fields!("setup_dev", step = "rewards skipped (may already exist)", error = e.to_string()),
        }
    }

    info_with_fields!("setup_dev", step = "add notifications");
    let notifications = vec![
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsEnabled.into(),
            metadata: None,
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::ReferralJoined.into(),
            metadata: Some(serde_json::json!({"username": "alice", "points": 100})),
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCodeDisabled.into(),
            metadata: None,
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsCreateUsername.into(),
            metadata: Some(serde_json::json!({"points": 50})),
        },
        NewNotificationRow {
            wallet_id: wallet.id,
            asset_id: None,
            notification_type: NotificationType::RewardsInvite.into(),
            metadata: Some(serde_json::json!({"username": "bob", "points": 200})),
        },
    ];
    let result = database.notifications()?.create_notifications(notifications)?;
    info_with_fields!("setup_dev", step = "notifications added", count = result);

    info_with_fields!("setup_dev", step = "add price alerts");
    let price_alerts = vec![
        PriceAlert::new_price(AssetId::from_chain(Chain::Ethereum), "USD".to_string(), 3000.0, PriceAlertDirection::Up),
        PriceAlert::new_price(AssetId::from_chain(Chain::Bitcoin), "USD".to_string(), 50000.0, PriceAlertDirection::Down),
    ];
    let result = database.price_alerts()?.add_price_alerts(&ios_device_id, price_alerts)?;
    info_with_fields!("setup_dev", step = "price alerts added", count = result);

    Ok(())
}

fn setup_dev_fiat_transactions(database: &Database, evm_address: &str, solana_address: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add fiat transactions");

    let mock = || FiatTransaction {
        asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
        transaction_type: FiatQuoteType::Buy,
        provider_id: FiatProviderName::MoonPay,
        provider_transaction_id: "setup-dev-mock".to_string(),
        status: FiatTransactionStatus::Pending,
        country: Some("US".to_string()),
        symbol: "ETH".to_string(),
        fiat_amount: 150.0,
        fiat_currency: "USD".to_string(),
        transaction_hash: None,
        address: Some(evm_address.to_string()),
    };

    let transactions = vec![
        FiatTransaction {
            provider_transaction_id: "setup-dev-moonpay-pending".to_string(),
            ..mock()
        },
        FiatTransaction {
            provider_id: FiatProviderName::Mercuryo,
            provider_transaction_id: "setup-dev-mercuryo-complete".to_string(),
            status: FiatTransactionStatus::Complete,
            fiat_amount: 320.5,
            transaction_hash: Some("0xsetupdevcomplete".to_string()),
            ..mock()
        },
        FiatTransaction {
            asset_id: Some(AssetId::from_chain(Chain::Solana)),
            transaction_type: FiatQuoteType::Sell,
            provider_id: FiatProviderName::Transak,
            provider_transaction_id: "setup-dev-transak-failed".to_string(),
            status: FiatTransactionStatus::Failed,
            symbol: "SOL".to_string(),
            fiat_amount: 95.25,
            address: Some(solana_address.to_string()),
            ..mock()
        },
    ];

    let mut fiat = database.fiat()?;
    let count = FiatRepository::add_fiat_transaction(&mut fiat, transactions)?;

    info_with_fields!("setup_dev", step = "fiat transactions added", count = count);
    Ok(())
}

fn setup_dev_assets(database: &Database) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info_with_fields!("setup_dev", step = "add assets");

    let assets = Chain::all().into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database.assets()?.add_assets(assets);

    info_with_fields!("setup_dev", step = "add fiat assets");

    let ethereum_asset_id = AssetId::from_chain(Chain::Ethereum).to_string();
    let smartchain_asset_id = AssetId::from_chain(Chain::SmartChain).to_string();

    let fiat_asset = |provider: &str, code: &str, symbol: &str, network: &str, asset_id: &str| FiatAssetRow {
        id: format!("{}_{}", provider, code),
        asset_id: Some(asset_id.to_string()),
        provider: provider.to_string(),
        code: code.to_string(),
        symbol: symbol.to_string(),
        network: Some(network.to_string()),
        token_id: None,
        is_enabled: true,
        is_enabled_by_provider: true,
        is_buy_enabled: true,
        is_sell_enabled: true,
        buy_limits: None,
        sell_limits: None,
        unsupported_countries: None,
    };

    let fiat_assets = vec![
        fiat_asset(FiatProviderName::MoonPay.as_ref(), "eth", "ETH", "ethereum", &ethereum_asset_id),
        fiat_asset(FiatProviderName::Mercuryo.as_ref(), "ETH", "ETH", "ETHEREUM", &ethereum_asset_id),
        fiat_asset(FiatProviderName::MoonPay.as_ref(), "bnb_bsc", "BNB", "binance_smart_chain", &smartchain_asset_id),
        fiat_asset(FiatProviderName::Mercuryo.as_ref(), "BNB", "BNB", "BINANCESMARTCHAIN", &smartchain_asset_id),
    ];

    let result = database.fiat()?.add_fiat_assets(fiat_assets)?;
    info_with_fields!("setup_dev", step = "fiat assets added", count = result);

    info_with_fields!("setup_dev", step = "add fiat provider countries");

    let fiat_countries: Vec<FiatProviderCountryRow> = FiatProviderName::all()
        .into_iter()
        .map(|provider| {
            let id = provider.id();
            FiatProviderCountryRow {
                id: format!("{}_us", id),
                provider: id.to_string(),
                alpha2: "US".to_string(),
                is_allowed: true,
            }
        })
        .collect();

    let result = database.fiat()?.add_fiat_providers_countries(fiat_countries)?;
    info_with_fields!("setup_dev", step = "fiat provider countries added", count = result);

    info_with_fields!("setup_dev", step = "add prices and charts");
    let now = chrono::Utc::now().naive_utc();
    let bitcoin_asset_id = AssetId::from_chain(Chain::Bitcoin).to_string();
    let coins: Vec<(&str, &str, f64)> = vec![
        (Chain::Bitcoin.as_ref(), &bitcoin_asset_id, 60000.0),
        (Chain::Ethereum.as_ref(), &ethereum_asset_id, 2000.0),
    ];

    let prices: Vec<PriceRow> = coins
        .iter()
        .map(|(coin_id, _, base_price)| PriceRow::with_price(coin_id.to_string(), *base_price))
        .collect();

    let price_assets: Vec<PriceAssetRow> = coins
        .iter()
        .map(|(coin_id, asset_id, _)| PriceAssetRow::new(asset_id.to_string(), coin_id.to_string()))
        .collect();

    let result = database.prices()?.set_prices(prices)?;
    info_with_fields!("setup_dev", step = "prices added", count = result);

    let result = database.prices()?.set_prices_assets(price_assets)?;
    info_with_fields!("setup_dev", step = "prices_assets added", count = result);

    for (idx, (coin_id, _, base_price)) in coins.iter().enumerate() {
        let seed = (idx + 1) as f64;
        let gen_price = |i: f64, scale: f64| (base_price + ((i * 0.3 + seed * 7.0).sin() + (i * 0.07).cos()) * base_price * scale).max(base_price * 0.1);

        let hourly: Vec<ChartRow> = (0i64..720)
            .map(|h| ChartRow::new(coin_id.to_string(), gen_price(h as f64, 0.1), now - chrono::Duration::hours(h)))
            .collect();

        let daily: Vec<ChartRow> = (30i64..1825)
            .map(|d| ChartRow::new(coin_id.to_string(), gen_price(d as f64, 0.15), now - chrono::Duration::days(d)))
            .collect();

        database.charts()?.add_charts_hourly(hourly)?;
        database.charts()?.add_charts_daily(daily)?;
    }
    info_with_fields!("setup_dev", step = "charts added");

    Ok(())
}
