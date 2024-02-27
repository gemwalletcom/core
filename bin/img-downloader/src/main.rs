use futures_util::StreamExt;
use pricer::{
    coingecko::{CoinGeckoClient, CoinInfo},
    price_mapper::get_chain_for_coingecko_id,
};
use primitives::ethereum_address::EthereumAddress;
use settings::Settings;
use std::{
    env, error::Error, fs, io::Write, path::Path, str::FromStr, thread::sleep, time::Duration,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let api_key = Settings::new().unwrap().coingecko.key.secret;
    let client = CoinGeckoClient::new(api_key);

    let default_path = "./coingecko".to_string();
    let target_path = args.get(1).unwrap_or(&default_path);
    println!("Save path: {}", target_path);

    let folder = Path::new(target_path);
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
    println!("process: {}", coin_id);
    if is_native_asset(&coin_info) {
        return handle_native_asset(&coin_info, folder).await;
    }

    for (platform, address) in coin_info.platforms.iter().filter(|(k, _)| !k.is_empty()) {
        let chain = get_chain_for_coingecko_id(platform);
        if chain.is_none() || address.is_empty() {
            // println!("<== {} not supported, skip", platform);
            continue;
        }

        let chain = chain.unwrap();
        let mut address_folder = address.clone();
        if chain.chain_type() == primitives::ChainType::Ethereum {
            address_folder = EthereumAddress::from_str(address)?.to_checksum();
        }
        let image_url = coin_info.image.large.clone();

        // build <folder>/ethereum/assets/<address>/logo.png
        let mut path = folder.join(chain.to_string());
        path.push("assets");
        path.push(address_folder.clone());
        if path.exists() {
            // println!("<== {}/{} already exists, skip", chain, &address_folder);
            return Ok(());
        }
        fs::create_dir_all(path.clone())?;

        path = path.join("logo.png");
        println!("==> download image for {}/{}", chain, address);
        download_image(&image_url, path.to_str().unwrap()).await?;

        // sleep for 300ms
        sleep(Duration::new(0, 300_000_000));
    }

    Ok(())
}

fn is_native_asset(coin_info: &CoinInfo) -> bool {
    coin_info.platforms.keys().filter(|p| !p.is_empty()).count() == 0
}

async fn handle_native_asset(coin_info: &CoinInfo, folder: &Path) -> Result<(), Box<dyn Error>> {
    let image_url = coin_info.image.large.clone();
    let chain = get_chain_for_coingecko_id(coin_info.id.as_str());
    if chain.is_none() {
        // println!("<== {} not supported, skip", coin_info.id);
        return Ok(());
    }
    let chain = chain.unwrap();

    // build <folder>/ethereum/info/logo.png
    let mut path = folder.join(chain.to_string());
    path.push("info");
    if path.exists() {
        // println!("<== {} native asset already exists, skip", chain);
        return Ok(());
    }
    fs::create_dir_all(path.clone())?;
    path = path.join("logo.png");
    println!("==> download image for native asset: {}", chain);
    download_image(&image_url, path.to_str().unwrap()).await?;
    // sleep for 300ms
    sleep(Duration::new(0, 300_000_000));
    Ok(())
}

async fn download_image(url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    if response.status() != 200 {
        return Err("<== image not found".into());
    }
    let mut file = fs::File::create(path)?;
    let mut stream = response.bytes_stream();
    while let Some(bytess) = stream.next().await {
        _ = file.write(&bytess?)?;
    }
    Ok(())
}
