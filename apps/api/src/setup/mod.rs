use primitives::{AddressType, Asset, AssetTag, AssetType, Chain, FiatProviderName, LinkType, NFTType, PlatformStore, TransactionType};
use search_index::{ASSETS_FILTERS, ASSETS_INDEX_NAME, ASSETS_RANKING_RULES, ASSETS_SEARCH_ATTRIBUTES, ASSETS_SORTS, INDEX_PRIMARY_KEY, SearchIndexClient};
use settings::Settings;
use storage::DatabaseClient;
use streamer::{ExchangeName, QueueName, StreamProducer};

pub async fn run_setup(settings: Settings) {
    println!("setup init");

    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);
    database_client.migrations().run_migrations().unwrap();
    println!("postgres migrations complete");

    let chains = Chain::all();

    println!("chains: {chains:?}");

    println!("setup add chains");
    let _ = database_client.assets().add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    println!("setup parser state");
    for chain in chains.clone() {
        let _ = database_client.parser_state().add_parser_state(chain.as_ref());
    }

    println!("setup assets_types");

    let assets_types = AssetType::all();
    let _ = database_client.assets_types().add_assets_types(assets_types);

    println!("setup assets");
    let assets = chains.into_iter().map(|x| Asset::from_chain(x).as_basic_primitive()).collect::<Vec<_>>();
    let _ = database_client.assets().add_assets(assets);

    println!("setup fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProvider::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.fiat().add_fiat_providers(providers);

    println!("setup releases");

    let releases = PlatformStore::all()
        .into_iter()
        .map(|x| storage::models::Release {
            platform_store: x.as_ref().to_string(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
        })
        .collect::<Vec<_>>();

    let _ = database_client.releases().add_releases(releases);

    let search_indexes = vec![ASSETS_INDEX_NAME];

    println!("setup nft types");
    let types = NFTType::all().into_iter().map(storage::models::NftType::from_primitive).collect::<Vec<_>>();
    let _ = database_client.nft().add_nft_types(types);

    println!("setup link types");
    let _ = database_client.link_types().add_link_types(LinkType::all());

    println!("setup scan address types");
    let address_types = AddressType::all()
        .into_iter()
        .map(storage::models::ScanAddressType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.scan_addresses().add_scan_address_types(address_types);

    println!("setup transaction types");
    let address_types = TransactionType::all()
        .into_iter()
        .map(storage::models::TransactionType::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.transactions().add_transactions_types(address_types);

    println!("setup assets tags");
    let assets_tags = AssetTag::all().into_iter().map(storage::models::Tag::from_primitive).collect::<Vec<_>>();
    let _ = database_client.tag().add_tags(assets_tags);

    println!("setup search index: {search_indexes:?}");

    let search_index_client = SearchIndexClient::new(&settings.meilisearch.url, settings.meilisearch.key.as_str());

    for index in search_indexes {
        search_index_client.create_index(index, INDEX_PRIMARY_KEY).await.unwrap();
    }
    let _ = search_index_client.set_filterable_attributes(ASSETS_INDEX_NAME, ASSETS_FILTERS.to_vec()).await;
    let _ = search_index_client.set_sortable_attributes(ASSETS_INDEX_NAME, ASSETS_SORTS.to_vec()).await;
    let _ = search_index_client
        .set_searchable_attributes(ASSETS_INDEX_NAME, ASSETS_SEARCH_ATTRIBUTES.to_vec())
        .await;
    let _ = search_index_client.set_ranking_rules(ASSETS_INDEX_NAME, ASSETS_RANKING_RULES.to_vec()).await;

    println!("setup queues");

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

    println!("setup complete");
}
