mod cli_args;
use cli_args::Args;
mod cli_model;

use coingecko::get_chain_for_coingecko_platform_id;
use coingecko::{CoinGeckoClient, CoinInfo};
use gem_evm::address::EthereumAddress;
use settings::Settings;

use clap::Parser;
use futures_util::StreamExt;
use std::{
    error::Error, fs, io::Write, path::Path, str::FromStr, thread::sleep, time::Duration, vec,
};

/// Assets image downloader from coingecko
struct Downloader {
    args: Args,
    client: CoinGeckoClient,
    ignore_chains: Vec<primitives::Chain>,
    cool_down: Duration,
}

impl Downloader {
    fn new(args: Args, api_key: String) -> Self {
        let client = CoinGeckoClient::new(api_key);
        let ignore_chains = vec![primitives::Chain::Binance];
        Self {
            args,
            client,
            ignore_chains,
            cool_down: Duration::new(0, 300_000_000),
        }
    }

    async fn start(&self) -> Result<(), Box<dyn Error>> {
        println!("==> Save path: {}", self.args.folder);
        let folder = Path::new(&self.args.folder);
        if !folder.exists() {
            fs::create_dir_all(folder)?;
        }

        if !self.args.coin_id.is_empty() {
            return self
                .handle_coin_id(self.args.coin_id.as_str(), folder)
                .await;
        }

        if !self.args.coin_ids.is_empty() {
            return self
                .handle_coin_ids(self.args.coin_ids.clone(), folder)
                .await;
        }

        if !self.args.coin_ids_url.is_empty() {
            return self.handle_coin_url(&self.args.coin_ids_url, folder).await;
        }

        self.handle_coingecto_top(folder).await
    }

    async fn handle_coin_id(&self, coin_id: &str, folder: &Path) -> Result<(), Box<dyn Error>> {
        self.handle_coin(coin_id, folder).await?;
        Ok(())
    }

    async fn handle_coin_ids(&self, coin_ids: String, folder: &Path) -> Result<(), Box<dyn Error>> {
        let ids: Vec<String> = coin_ids.split(',').map(|x| x.trim().to_string()).collect();
        for coin_id in ids {
            self.handle_coin(&coin_id, folder).await?;
        }
        Ok(())
    }

    async fn handle_coin_url(&self, url: &str, folder: &Path) -> Result<(), Box<dyn Error>> {
        let response: cli_model::Response = reqwest::get(url).await?.json().await?;
        for price in response.results {
            match self.handle_coin_id(&price.coin_id, folder).await {
                Ok(_) => continue,
                Err(e) => println!("<== {} error: {}", price.coin_id, e),
            }
        }
        Ok(())
    }

    async fn handle_coingecto_top(&self, folder: &Path) -> Result<(), Box<dyn Error>> {
        let mut page = self.args.page;
        let total_pages = self.args.count.div_ceil(self.args.page_size);
        while page <= total_pages {
            let markets = self
                .client
                .get_coin_markets(page, self.args.page_size)
                .await?;
            for market in markets {
                self.handle_coin(&market.id, folder).await?;
            }
            page += 1;
        }
        Ok(())
    }

    async fn handle_coin(&self, coin_id: &str, folder: &Path) -> Result<(), Box<dyn Error>> {
        println!("==> process: {}", coin_id);
        let coin_info = self.client.get_coin(coin_id).await?;
        if self.is_native_asset(&coin_info) {
            return Ok(());
        }

        for (platform, address) in coin_info.platforms.iter().filter(|(k, _)| !k.is_empty()) {
            let chain = get_chain_for_coingecko_platform_id(platform);
            if chain.is_none() || address.is_empty() {
                if self.args.verbose {
                    println!("<== {} not supported, skip", platform);
                }
                continue;
            }

            let chain = chain.unwrap();
            if self.ignore_chains.contains(&chain) {
                continue;
            }

            if let Some(denom) = chain.as_denom() {
                if denom == address {
                    if self.args.verbose {
                        println!("<== skip native denom: {}", denom);
                    }
                    continue;
                }
            }
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
                if self.args.verbose {
                    println!("<== {:?} already exists, skip", &path);
                }
                return Ok(());
            }
            fs::create_dir_all(path.clone())?;

            path = path.join("logo.png");
            println!("==> download image for {}/{}", chain, address);
            crate::download_image(&image_url, path.to_str().unwrap()).await?;

            sleep(self.cool_down);
        }

        Ok(())
    }

    fn is_native_asset(&self, coin_info: &CoinInfo) -> bool {
        coin_info.platforms.keys().filter(|p| !p.is_empty()).count() == 0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli_args::Args::parse();
    let api_key = Settings::new().unwrap().coingecko.key.secret;
    let downloader = Downloader::new(args, api_key);

    downloader.start().await
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
