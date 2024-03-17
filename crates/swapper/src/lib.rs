pub mod client;
use oneinch::OneInchClient;

pub use self::client::SwapperClient;
pub mod jupiter;
pub use self::jupiter::JupiterClient;
pub mod thorswap;
pub use self::thorswap::ThorchainSwapClient;

pub struct SwapperConfiguration {
    pub oneinch: SwapperClientConfiguration,
    pub jupiter: SwapperClientConfiguration,
    pub thorchain: SwapperClientConfiguration,
}

pub struct SwapperClientConfiguration {
    pub url: String,
    pub key: String,
    pub fee_percent: f64,
    pub fee_address: String,
}

pub struct SwapperOneinchConfiguration {}

pub struct Swapper {}

impl Swapper {
    pub fn build(configuration: SwapperConfiguration) -> SwapperClient {
        let oneinch_client = OneInchClient::new(
            configuration.oneinch.url,
            configuration.oneinch.key,
            configuration.oneinch.fee_percent,
            configuration.oneinch.fee_address,
        );
        let jupiter_client = JupiterClient::new(
            configuration.jupiter.url,
            configuration.jupiter.fee_percent,
            configuration.jupiter.fee_address,
        );
        let thorchain_swap_client = ThorchainSwapClient::new(
            configuration.thorchain.url,
            configuration.thorchain.fee_percent,
            configuration.thorchain.fee_address,
        );
        SwapperClient::new(oneinch_client, jupiter_client, thorchain_swap_client)
    }
}
