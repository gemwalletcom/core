use pricer::ClickhouseDatabase;
use primitives::{Chain, Asset};
use settings::Settings;
use storage::DatabaseClient;

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

    let chains = settings.parser.chains.into_iter().filter_map(|chain| Chain::from_str(chain.as_str())).collect::<Vec<_>>();

    println!("chains: {:?}", chains);

    println!("setup parser state");
    for chain in chains.clone() {
        let _ = database_client.add_parser_state(chain);
    }

    println!("setup assets");
    let assets = chains.into_iter()
        .map(Asset::from_chain)
        .map(storage::models::Asset::from_primitive)
        .collect::<Vec<_>>();
    let _ = database_client.add_assets(assets);

    println!("setup complete");
}