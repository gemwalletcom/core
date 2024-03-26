use clap::{arg, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to save images
    #[arg(short, long)]
    pub folder: String,

    /// Top tokens on coingecko to download
    #[arg(short, long, default_value_t = 50)]
    pub count: u32,

    /// Starting page for coingecko api
    #[arg(short, long, default_value_t = 1)]
    pub page: u32,

    /// Page size for coingecko api
    #[arg(long, default_value_t = 50)]
    pub page_size: u32,

    /// ID of the coin, if this is set, it will only download the image for the coin
    #[arg(long, default_value = "")]
    pub coin_id: String,

    /// Coin IDs separated by comma to download, exclusive with coin_ids_url
    #[arg(long, default_value = "")]
    pub coin_ids: String,

    /// Coin IDs from URL to download, exclusive with coin_ids_url
    #[arg(long, default_value = "")]
    pub coin_ids_url: String,

    /// Verbose mode
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
