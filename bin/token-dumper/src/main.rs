use futures_util::StreamExt;
use pricer::coingecko::CoinGeckoClient;
use std::{env, error::Error, fs, io::Write, path::Path, thread::sleep, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let default_path = "./coingecko".to_string();
    let target_path = args.get(1).unwrap_or(&default_path);
    println!("Save path: {}", target_path);

    let folder = Path::new(target_path);
    let api_key = std::env::var("COINGECKO_KEY_SECRET").expect("COINGECKO_KEY_SECRET not set");
    let client = CoinGeckoClient::new(api_key);
    if !folder.exists() {
        fs::create_dir_all(folder)?;
    }

    let mut page = 1;
    let total_pages = 10;
    let page_size = 100;
    while page <= total_pages {
        let markets = client.get_coin_markets(page, page_size).await?;
        for market in markets {
            // FIXME run in parallel
            handle_coin(&market.id, &client, folder).await?;
        }
        page += 1;
    }
    Ok(())
}

async fn handle_coin(
    coin_id: &str,
    client: &CoinGeckoClient,
    folder: &Path,
) -> Result<(), Box<dyn Error>> {
    let coin_info = client.get_coin(coin_id).await?;
    let image_url = coin_info.image.large;
    let mut path = folder.join(coin_id);
    if path.exists() {
        println!("{} already exists, skip", coin_id);
        return Ok(());
    }
    fs::create_dir_all(path.clone())?;
    path = path.join("image.png");
    println!("download image for {}", coin_id);
    download_image(&image_url, path.to_str().unwrap()).await?;
    sleep(Duration::new(0, 300_000_000));
    Ok(())
}

async fn download_image(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    if response.status() != 200 {
        return Err("image not found".into());
    }
    let mut file = fs::File::create(path)?;
    let mut stream = response.bytes_stream();
    while let Some(bytess) = stream.next().await {
        _ = file.write(&bytess?)?;
    }
    Ok(())
}
