mod cli_args;
use cli_args::Args;

use coingecko::get_chain_for_coingecko_platform_id;
use coingecko::{CoinGeckoClient, CoinInfo};
use settings::Settings;

use clap::Parser;
use futures_util::StreamExt;
use reqwest::{retry, StatusCode};
use std::{error::Error, fs, io::Write, path::Path, thread::sleep, time::Duration};

/// Assets image downloader from coingecko
struct Downloader {
    args: Args,
    client: CoinGeckoClient,
}

impl Downloader {
    fn new(args: Args, api_key: String) -> Self {
        let client = Self::new_coingecko_client(api_key);
        Self { args, client }
    }

    fn new_coingecko_client(api_key: String) -> CoinGeckoClient {
        let retry_policy = retry::for_host("api.coingecko.com")
            .max_retries_per_request(10)
            .classify_fn(|req_rep| {
                match req_rep.status() {
                    Some(StatusCode::TOO_MANY_REQUESTS) | 
                    Some(StatusCode::INTERNAL_SERVER_ERROR) |
                    Some(StatusCode::BAD_GATEWAY) |
                    Some(StatusCode::SERVICE_UNAVAILABLE) |
                    Some(StatusCode::GATEWAY_TIMEOUT) => req_rep.retryable(),
                    None => req_rep.retryable(), // Network errors
                    _ => req_rep.success(),
                }
            });
        
        let reqwest_client = reqwest::Client::builder()
            .retry(retry_policy)
            .build()
            .expect("Failed to build reqwest client");
            
        CoinGeckoClient::new_with_reqwest_client(reqwest_client, api_key.as_str())
    }

    async fn start(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("==> Save path: {}", self.args.folder);
        let folder = Path::new(&self.args.folder);
        if !folder.exists() {
            fs::create_dir_all(folder)?;
        }

        if !self.args.coin_id.is_empty() {
            return self.handle_coin_id(self.args.coin_id.as_str(), folder).await;
        }

        if !self.args.coin_ids.is_empty() {
            return self.handle_coin_ids(self.coin_ids(self.args.coin_ids.clone()), folder).await;
        }

        if !self.args.coin_list.is_empty() {
            return self.handle_coin_list(self.args.coin_list.clone(), folder).await;
        }

        unimplemented!("specify coin_id, coin_ids or coin_list")
    }

    fn coin_ids(&self, list: String) -> Vec<String> {
        list.split(',').map(|x| x.trim().to_string()).collect()
    }
    async fn handle_coin_list(&self, list: String, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ids = match list.as_str() {
            "trending" => self.client.get_search_trending().await?.get_coins_ids(),
            "top" => self.get_coingecko_top().await?,
            "new" => self.client.get_coin_list_new().await?.ids().iter().take(20).cloned().collect(),
            _ => {
                vec![]
            }
        };
        self.handle_coin_ids(ids, folder).await
    }

    async fn handle_coin_ids(&self, coin_ids: Vec<String>, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        for coin_id in coin_ids {
            self.handle_coin_id(&coin_id, folder).await?;
            sleep(Duration::from_millis(self.args.delay.into()));
        }
        Ok(())
    }

    async fn get_coingecko_top(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut page = self.args.page;
        let total_pages = self.args.count.div_ceil(self.args.page_size);
        let mut ids: Vec<String> = Vec::new();

        while page <= total_pages && page > 0 {
            let markets = self.client.get_coin_markets(page, self.args.page_size).await?;
            for market in markets {
                ids.push(market.id.clone());
            }
            page += 1;
        }
        Ok(ids)
    }

    async fn handle_coin_id(&self, coin_id: &str, folder: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("==> process: {coin_id}");
        let coin_info = self.client.get_coin(coin_id).await?;
        if self.is_native_asset(&coin_info) {
            return Ok(());
        }

        for (platform, address) in coin_info.platforms.iter().filter(|(k, _)| !k.is_empty()) {
            let chain = get_chain_for_coingecko_platform_id(platform);
            if chain.is_none() || address.is_empty() {
                if self.args.verbose {
                    println!("<== {platform} not supported, skip");
                }
                continue;
            }

            let chain = chain.unwrap();

            if let Some(denom) = chain.as_denom() {
                if denom == address {
                    if self.args.verbose {
                        println!("<== skip native denom: {denom}");
                    }
                    continue;
                }
            }

            let image_url: String = coin_info.image.large.clone();
            if let Some(address_folder) = chain_primitives::format_token_id(chain, address.clone()) {
                // build <folder>/ethereum/assets/<address>/logo.png
                let mut path = folder.join(chain.to_string());
                path.push("assets");
                path.push(address_folder.clone());
                if path.exists() {
                    if self.args.verbose {
                        println!("<== {:?} already exists, skip", &path);
                    }
                    return Ok(());
                }
                fs::create_dir_all(path.clone())?;

                path = path.join("logo.png");
                println!("==> download image for {chain}/{address}");
                println!("==> image url: {image_url}");
                crate::download_image(&image_url, path.to_str().unwrap()).await?;

                sleep(Duration::from_millis(self.args.delay.into()));
            }
        }

        Ok(())
    }

    fn is_native_asset(&self, coin_info: &CoinInfo) -> bool {
        coin_info.platforms.keys().filter(|p| !p.is_empty()).count() == 0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = cli_args::Args::parse();
    let api_key = Settings::new().unwrap().coingecko.key.secret;
    let downloader = Downloader::new(args, api_key);

    downloader.start().await
}

async fn download_image(url: &str, path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
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
