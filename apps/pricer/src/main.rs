use settings::Settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("pricer init");

    let _api_key = Settings::new().unwrap().coingecko.key.secret;

    Ok(())
}
