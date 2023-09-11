pub mod pusher;
pub mod parser;
pub mod provider;

use primitives::Chain;
use settings::Settings;
use storage::DatabaseClient;

use crate::{pusher::Pusher, parser::{Parser, ParserOptions}};

#[tokio::main]
pub async fn main() {
    println!("parser init");
    let settings: Settings = Settings::new().unwrap();
    let chain_string = std::env::args().nth(1).unwrap_or(settings.parser.chain.clone());
    let chain = Chain::from_str(chain_string.as_str()).expect("parser invalid chain specified");
    let provider = provider::new(chain, &settings);
    let pusher = Pusher::new(
        settings.pusher.url.clone(),
        settings.postgres.url.clone(),
        settings.pusher.ios.topic.clone(),
    );
    let database_client = DatabaseClient::new(settings.postgres.url.as_str());
    let options = ParserOptions{
        timeout: settings.parser.timeout,
    };
    let mut parser = Parser::new(
        &*provider, 
        pusher, 
        database_client,
        options,
    );
    println!("parser start chain: {}", provider.get_chain().as_str());

    parser.start().await;
}