use primitives::{Asset, AssetType, Chain, FiatProviderName, PlatformStore};
use settings::Settings;
use storage::{ClickhouseDatabase, DatabaseClient};

#[tokio::main]
async fn main() {
    println!("setup init");

    let settings = Settings::new().unwrap();

    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);
    database_client.migrations();
    println!("postgres migrations complete");

    let clickhouse_database = ClickhouseDatabase::new(&settings.clickhouse.url);
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

    println!("setup complete");
}
