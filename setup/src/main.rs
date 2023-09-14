use settings::Settings;
use storage::DatabaseClient;

#[tokio::main]
async fn main() {
    println!("setup init");

    let settings = Settings::new().unwrap();

    let postgres_url = settings.postgres.url.as_str();
    let mut database_client: DatabaseClient = DatabaseClient::new(postgres_url);    
    database_client.migrations();

    println!("setup complete");
}