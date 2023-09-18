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

    let chains = settings.parser.chains.into_iter().map(|chain| Chain::from_str(chain.as_str())).flatten().collect::<Vec<_>>();

    println!("setup parser state");
    for chain in chains.clone() {
        let _ = database_client.add_parser_state(chain);
    }

    println!("setup assets");
    let assets = chains.into_iter()
        .map(|x| Asset::from_chain(x))
        .map(|x| storage::models::Asset::from_primitive(x))
        .collect::<Vec<_>>();
    let _ = database_client.add_assets(assets);

    println!("setup complete");
}