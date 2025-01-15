use primitives::{Asset, AssetType, Chain, FiatProviderName, NFTType, PlatformStore};
use search_index::{SearchIndexClient, ASSETS_FILTERS, ASSETS_INDEX_NAME, ASSETS_RANKING_RULES, ASSETS_SEARCH_ATTRIBUTES, ASSETS_SORTS, INDEX_PRIMARY_KEY};
use settings::Settings;
use storage::{ClickhouseClient, DatabaseClient};

#[tokio::main]
async fn main() {
    println!("setup init");

    let settings = Settings::new().unwrap();

    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);
    database_client.migrations();
    println!("postgres migrations complete");

    let clickhouse_database = ClickhouseClient::new(&settings.clickhouse.url, &settings.clickhouse.database);
    let _ = clickhouse_database.migrations().await;
    println!("clickhouse migrations complete");

    let chains = Chain::all();

    println!("chains: {:?}", chains);

    println!("setup add chains");
    let _ = database_client.add_chains(chains.clone().into_iter().map(|x| x.to_string()).collect());

    println!("setup parser state");
    for chain in chains.clone() {
        let _ = database_client.add_parser_state(chain);
    }

    println!("setup assets_types");

    let assets_types = AssetType::all();
    let _ = database_client.add_assets_types(assets_types);

    println!("setup assets");
    let assets = chains
        .into_iter()
        .map(Asset::from_chain)
        .map(storage::models::Asset::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.add_assets(assets);

    println!("setup fiat providers");
    let providers = FiatProviderName::all()
        .into_iter()
        .map(storage::models::FiatProvider::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.add_fiat_providers(providers);

    println!("setup releases");

    let releases = PlatformStore::all()
        .into_iter()
        .map(|x| storage::models::Release {
            platform_store: x.as_ref().to_string(),
            version: "1.0.0".to_string(),
            upgrade_required: false,
        })
        .collect::<Vec<_>>();

    let _ = database_client.add_releases(releases);

    let search_indexes = vec![ASSETS_INDEX_NAME];

    println!("setup nft types");
    let types = NFTType::all().into_iter().map(storage::models::NftType::from_primitive).collect::<Vec<_>>();
    let _ = database_client.add_nft_types(types);

    println!("setup search index: {:?}", search_indexes);

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

    println!("setup complete");
}
