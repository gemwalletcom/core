use primitives::Chain;
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
    for chain in chains {
        let _ = database_client.add_parser_state(chain);
    }

    println!("setup complete");
}