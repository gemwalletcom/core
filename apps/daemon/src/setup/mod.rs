use gem_tracing::info_with_fields;
use primitives::{AddressType, Asset, AssetTag, AssetType, Chain, FiatProviderName, LinkType, NFTType, PlatformStore, TransactionType};
use search_index::{INDEX_CONFIGS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use storage::DatabaseClient;
use streamer::{ExchangeName, QueueName, StreamProducer};

pub async fn run_setup(settings: Settings) {
    info_with_fields!("setup", step = "init");

    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);
    database_client.migrations().run_migrations().unwrap();
    info_with_fields!("setup", step = "postgres migrations complete");

    let chains = Chain::all();

    info_with_fields!("setup", step = "chains", chains = format!("{:?}", chains));

    info_with_fields!("setup", step = "add chains");
    let _ = database_client.assets().add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    info_with_fields!("setup", step = "parser state");
    for chain in chains.clone() {
        let _ = database_client.parser_state().add_parser_state(chain.as_ref());
    }

    info_with_fields!("setup", step = "assets_types");

    let assets_types = AssetType::all();
    let _ = database_client.assets_types().add_assets_types(assets_types);

    info_with_fields!("setup", step = "assets");
    let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database_client.assets().add_assets(assets);

    info_with_fields!("setup", step = "fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProvider::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.fiat().add_fiat_providers(providers);

    info_with_fields!("setup", step = "releases");

    let releases = PlatformStore::all()
        .into_iter()
        .map(|x| storage::models::Release {
            platform_store: x.as_ref().to_string(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
        })
        .collect::<Vec<_>>();

    let _ = database_client.releases().add_releases(releases);

    info_with_fields!("setup", step = "nft types");
    let types = NFTType::all().into_iter().map(storage::models::NftType::from_primitive).collect::<Vec<_>>();
    let _ = database_client.nft().add_nft_types(types);

    info_with_fields!("setup", step = "link types");
    let _ = database_client.link_types().add_link_types(LinkType::all());

    info_with_fields!("setup", step = "scan address types");
    let address_types = AddressType::all()
        .into_iter()
        .map(storage::models::ScanAddressType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.scan_addresses().add_scan_address_types(address_types);

    info_with_fields!("setup", step = "transaction types");
    let address_types = TransactionType::all()
        .into_iter()
        .map(storage::models::TransactionType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.transactions().add_transactions_types(address_types);

    info_with_fields!("setup", step = "assets tags");
    let assets_tags = AssetTag::all().into_iter().map(storage::models::Tag::from_primitive).collect::<Vec<_>>();
    let _ = database_client.tag().add_tags(assets_tags);

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
                QueueName::FetchTokenAddressesAssociations,
                QueueName::FetchCoinAddressesAssociations,
                QueueName::FetchTransactions,
                QueueName::FetchNftAssetsAddressesAssociations,
            ],
        )
        .await;

    info_with_fields!("setup", step = "complete");
}
