use clap::{arg, Parser};
use coingecko::get_chain_for_coingecko_id;
use coingecko::{CoinGeckoClient, CoinInfo};
use futures_util::StreamExt;
use gem_evm::address::EthereumAddress;
use settings::Settings;
use std::{
    error::Error, fs, io::Write, path::Path, str::FromStr, thread::sleep, time::Duration, vec,
};

/// Assets image downloader from coingecko
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to save images
    #[arg(short, long)]
    folder: String,

    /// Top number of tokens to download
    #[arg(short, long, default_value_t = 50)]
    count: u32,

    /// Starting page
    #[arg(short, long, default_value_t = 1)]
    page: u32,

    /// Page size
    #[arg(long, default_value_t = 50)]
    page_size: u32,

    /// ID of the coin
    #[arg(long)]
    coin_id: String,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

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

        let coin_id = self.args.coin_id.clone();
        if !coin_id.is_empty() {
            let market = self.client.get_coin_markets_id(coin_id.as_str()).await?;
            self.handle_coin(&market.clone().id, folder).await?;

            return Ok(());
        }

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
            let chain = get_chain_for_coingecko_id(platform);
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
    let args = Args::parse();
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
